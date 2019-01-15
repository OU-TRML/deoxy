//! Communication utilities.
use crate::actix::*;
use crate::{
    Action, Motor, MotorId, MotorMessage, Program, Protocol, Pump, PumpMessage, Step,
    ValidateProtocolError,
};

use std::ops::Index;

type Result<T> = std::result::Result<T, Error>;
type CoordContext = Context<Coordinator>;

/// **Expand for important information.**
///
/// Represents an error encountered by the coordinator.
///
/// # Important
///
/// Due to the critical nature of the coordinator, there will be a lot of possible variants.
/// Please *do not* use a wildcard match for expedience; this has real safety implications.
///
/// The responsibility for user safety falls on you as the library consumer. This library will of
/// course attempt to provide information about errors without crashing, but even this cannot be
/// guaranteed, so program defensively.
///
/// Once ([rust-lang/rust-clippy#3652](https://github.com/rust-lang/rust-clippy/issues/3652)) is
/// merged, you are advised to use that lint to enforce this with tooling.
#[derive(Clone, Copy, Debug)]
pub enum Error {
    /// An error was encountered in converting a protocol to a program.
    ProtocolConversion(ValidateProtocolError),
}

impl From<ValidateProtocolError> for Error {
    fn from(err: ValidateProtocolError) -> Self {
        Error::ProtocolConversion(err)
    }
}

#[derive(Clone, Copy, Debug)]
enum Message {
    /// The user has instructed us to move on to the next step.
    Continue,
    /// We have been asked to immediately stop the program.
    Halt,
    /// We have been asked to stop the program after the next step.
    ///
    /// The sample will be left in whatever buffer it is in; to resuspend it in a different buffer,
    /// use [`ExchangeStop`](#variant.ExchangeStop).
    Stop,
    /// We have been asked to finish this step, exchange the buffer, and stop.
    ExchangeStop(MotorId),
}

impl ActixMessage for Message {
    type Result = Result<()>;
}

/// Represents a coordinator state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum State {
    /// The coordinator is waiting for user input.
    Waiting,
    /// The coordinator has stopped and is waiting for further instruction.
    Stopped {
        /// Whether execution stopped early (was aborted).
        early: bool,
    },
    /// The program is actively executing.
    Running,
}

/// Contains communication necessities.
#[derive(Debug)]
struct Addresses {
    /// The addresses of each motor.
    motors: Vec<Addr<Motor>>,
    /// The address of the pump.
    pump: Addr<Pump>,
}

impl Index<MotorId> for Addresses {
    type Output = Addr<Motor>;
    /// Returns the address of the motor associated with the given buffer.
    fn index(&self, i: MotorId) -> &Self::Output {
        &self.motors[i]
    }
}

/// Contains program and buffer states.
#[derive(Debug)]
struct CoordState {
    /// The currently-in-progress (original) program.
    program: Option<Program>,
    /// The steps remaining, derived from the original program.
    remaining: Vec<Action>,
    /// The step we're currently running.
    current: Option<Action>,
    /// The most recent buffer.
    buffer: Option<MotorId>,
    /// The current status of program execution.
    status: State,
}

/// Contains all the actual logic for controlling the system based on a specified program.
#[derive(Debug)]
pub struct Coordinator {
    /// The pump driving everything.
    pump: Pump,
    /// The motors connected to various valves.
    motors: Vec<Motor>,
    /// The handles giving us access to everything.
    addresses: Addresses,
    /// Encodes the state of the coordinator.
    state: CoordState,
}

impl Coordinator {
    /// The in-progress program, if appropriate.
    pub fn program(&self) -> Option<&Program> {
        self.state.program.as_ref()
    }
    /// The current status of the coordinator.
    pub fn status(&self) -> State {
        self.state.status
    }
    /// Closes all valves.
    fn close_all(&self) {
        for addr in &self.addresses.motors {
            addr.do_send(MotorMessage::Close);
        }
    }
    /// Moves to the next step of the protocol, returning the new current action.
    fn advance(&mut self, context: &mut CoordContext) -> Result<Option<Action>> {
        if !self.state.remaining.is_empty() {
            let action = self.state.remaining.remove(0);
            // Make sure to message something that will call advance again later!
            match action {
                Action::Perfuse(_buffer) => unimplemented!(),
                Action::Sleep(duration) => {
                    context.run_later(duration, |coord, context| {
                        coord.advance(context).unwrap();
                    });
                }
                Action::Hail => self.state.status = State::Waiting,
                Action::Drain => unimplemented!(),
                Action::Finish => unimplemented!(),
            }
            self.state.current = Some(action);
            Ok(self.state.current)
        } else {
            self.state.status = State::Stopped { early: false };
            Ok(None)
        }
    }
    /// Clears the remaining program queue after the next perfusion.
    fn clear(&mut self) -> Result<()> {
        if let Some(index) = self
            .state
            .remaining
            .iter()
            .position(|action| action.is_disjoint())
        {
            // Vec::truncate keeps n elements, but we don't want to keep the element at index.
            self.state.remaining.truncate(index);
        }
        // TODO: Instead of doing this, mark the program as partially completed.
        self.state.program = None;
        Ok(())
    }
    /// Stop the program after the current step.
    ///
    /// If an end buffer is given, the current buffer will be replaced with that one before
    /// stopping (if necessary).
    fn stop<I>(&mut self, buffer: I) -> Result<()>
    where
        I: Into<Option<MotorId>>,
    {
        if let Some(target) = buffer.into() {
            if let Some(current) = self.state.buffer {
                if current == target {
                    // We're already in the target buffer; we don't need to do much else.
                    self.clear()?;
                } else {
                    let program = Protocol::with_step(Step::Perfuse(target, None)).as_program()?;
                    self.state.program = Some(program.clone());
                    self.state.remaining = program.into();
                }
            }
        }
        Ok(())
    }
    /// Continue the program.
    fn resume(&mut self, context: &mut CoordContext) -> Result<()> {
        if self.status() != State::Waiting {
            // TODO: Should we error instead of ignoring?
            log::warn!("Coordinator told to resume while not paused; ignoring.");
            return Ok(());
        }
        self.state.status = State::Running;
        self.advance(context)?;
        Ok(())
    }
    /// Abort the program no matter where we are.
    fn hcf(&mut self) -> Result<()> {
        self.addresses.pump.do_send(PumpMessage::Stop);
        self.close_all();
        // TODO: Should calling this method always send an error upstream?
        Ok(())
    }
}

impl Actor for Coordinator {
    type Context = CoordContext;
}

impl Handle<Message> for Coordinator {
    type Result = Result<()>;
    fn handle(&mut self, message: Message, context: &mut Self::Context) -> Self::Result {
        match message {
            Message::Continue => self.resume(context),
            Message::Stop => self.stop(None),
            Message::Halt => self.hcf(),
            Message::ExchangeStop(id) => self.stop(id),
        }
    }
}
