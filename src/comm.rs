//! Communication utilities.
use crate::actix::*;
use crate::{Action, Motor, MotorId, Program, Protocol, Pump, Step, ValidateProtocolError};

type Result<T> = std::result::Result<T, Error>;

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

/// Contains all the actual logic for controlling the system based on a specified program.
#[derive(Debug)]
pub struct Coordinator {
    /// The pump driving everything.
    pump: Pump,
    /// The motors connected to various valves.
    motors: Vec<Motor>,
    /// The currently-in-progress (original) program.
    program: Option<Program>,
    /// The iterator we're using, derived from the original program.
    iter: Option<<Program as IntoIterator>::IntoIter>,
    /// The step we're currently running.
    current: Option<Action>,
    /// The most recent buffer.
    buffer: Option<MotorId>,
    /// The current status of program execution.
    status: State,
}

impl Coordinator {
    /// The in-progress program, if appropriate.
    pub fn program(&self) -> Option<&Program> {
        self.program.as_ref()
    }
    /// The current status of the coordinator.
    pub fn status(&self) -> State {
        self.status
    }
    /// Clears the remaining program queue after the next perfusion.
    fn clear(&mut self) -> Result<()> {
        if let Some(iter) = &self.iter {
            let mut remaining = iter.clone().collect::<Vec<_>>();
            if let Some(index) = remaining.iter().position(|action| action.is_disjoint()) {
                // Vec::truncate keeps n elements, but we don't want to keep the element at index.
                remaining.truncate(index);
            }
            // TODO: Instead of doing this, mark the program as partially completed.
            self.program = None;
            Ok(())
        } else {
            Ok(())
        }
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
            if let Some(current) = self.buffer {
                if current == target {
                    // We're already in the target buffer; we don't need to do much else.
                    self.clear()?;
                } else {
                    let program = Protocol::with_step(Step::Perfuse(target, None)).as_program()?;
                    self.program = Some(program.clone());
                    self.iter = Some(program.into_iter());
                }
            }
        }
        unimplemented!()
    }
    /// Continue the program.
    fn resume(&mut self) -> Result<()> {
        if self.status() != State::Waiting {
            log::warn!("Coordinator told to resume while not paused; ignoring.");
            return Ok(());
        }
        unimplemented!()
    }
    /// Abort the program no matter where we are.
    fn hcf(&mut self) -> Result<()> {
        unimplemented!()
    }
}

impl Actor for Coordinator {
    type Context = Context<Self>;
}

impl Handle<Message> for Coordinator {
    type Result = Result<()>;
    fn handle(&mut self, message: Message, _context: &mut Self::Context) -> Self::Result {
        match message {
            Message::Continue => self.resume(),
            Message::Stop => self.stop(None),
            Message::Halt => self.hcf(),
            Message::ExchangeStop(id) => self.stop(id),
        }
    }
}
