//! Communication utilities.
use crate::actix::*;
use crate::{Motor, MotorId, Program, Pump};

type Result = std::result::Result<(), ()>;

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
    type Result = Result;
}

/// Represents a coordinator state.
#[derive(Clone, Copy, Debug)]
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
    current: Option<<Program as IntoIterator>::IntoIter>,
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
    /// Stop the program after the current step.
    ///
    /// If an end buffer is given, the current buffer will be replaced with that one before
    /// stopping (if necessary).
    fn stop<I>(&mut self, buffer: I) -> Result
    where
        I: Into<Option<MotorId>>,
    {
        unimplemented!()
    }
    /// Continue the program.
    fn resume(&mut self) -> Result {
        unimplemented!()
    }
    /// Abort the program no matter where we are.
    fn hcf(&mut self) -> Result {
        unimplemented!()
    }
}

impl Actor for Coordinator {
    type Context = Context<Self>;
}

impl Handle<Message> for Coordinator {
    type Result = Result;
    fn handle(&mut self, message: Message, _context: &mut Self::Context) -> Self::Result {
        match message {
            Message::Continue => self.resume(),
            Message::Stop => self.stop(None),
            Message::Halt => self.hcf(),
            Message::ExchangeStop(id) => self.stop(id),
        }
    }
}
