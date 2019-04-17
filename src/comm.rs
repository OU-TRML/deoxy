//! Communication utilities.
use crate::actix::*;
use crate::{
    mail, Action, Config, Motor, MotorId, MotorMessage, PinError, Program, Protocol, Pump,
    PumpMessage, Step, ValidateProtocolError,
};

use lazy_static::lazy_static;
use uom::si::f64::*;
use uom::si::time::second;
use uom::si::volume::milliliter;
use uom::si::volume_rate::milliliter_per_second;
use uuid::Uuid;

use std::{fmt, ops::Index, time::Duration};

lazy_static! {
    static ref VOLUME: Volume = Volume::new::<milliliter>(500.0);
    static ref RATE: VolumeRate = VolumeRate::new::<milliliter_per_second>(3.5);
    static ref TIME: Time = *VOLUME / *RATE;
    static ref DURATION: Duration = {
        let secs = TIME.get::<second>();
        let nanos = ((secs - secs.floor()) * 1.0_E9).floor() as u32;
        let secs = secs.floor() as u64;
        Duration::new(secs, nanos)
    };
}

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
#[derive(Debug)]
pub enum Error {
    /// An error was encountered in converting a protocol to a program.
    ProtocolConversion(ValidateProtocolError),
    /// We tried to start a new protocol while one was already running.
    Busy,
    /// A pin-related initialization error occured.
    Pin(PinError),
}

impl From<ValidateProtocolError> for Error {
    fn from(err: ValidateProtocolError) -> Self {
        Error::ProtocolConversion(err)
    }
}

impl From<PinError> for Error {
    fn from(err: PinError) -> Self {
        Error::Pin(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Coordinator error: {:?}", self)
    }
}

impl std::error::Error for Error {}

/// A message sent to control the coordinator.
#[derive(Debug)]
pub enum Message {
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
    /// The user has instructed us to start a new protocol.
    ///
    /// If the second parameter is specified, it is used as the label for the job; otherwise, one
    /// is generated.
    Start(Protocol, Option<Uuid>),
    /// Used to subscribe to coordinator updates.
    Subscribe(Box<dyn Update>),
}

impl ActixMessage for Message {
    type Result = Result<()>;
}

/// Represents a coordinator state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "use_serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "use_serde", serde(rename_all = "lowercase"))]
pub enum State {
    /// The coordinator is waiting for user input.
    Waiting,
    /// The coordinator has stopped and is waiting for further instruction.
    ///
    /// This state is the default if the coordinator has not yet run a program.
    Stopped {
        /// Whether execution stopped early (was aborted).
        early: bool,
    },
    /// The program is actively executing.
    Running,
}

impl Default for State {
    fn default() -> Self {
        State::Stopped { early: false }
    }
}

/// Contains communication necessities.
#[derive(Debug)]
struct Addresses {
    /// The addresses of each motor.
    motors: Vec<Addr<Motor>>,
    /// The address of the pump.
    pump: Addr<Pump>,
    /// The address of the subscriber entry point.
    subscribers: Addr<Subscribers>,
}

impl Index<MotorId> for Addresses {
    type Output = Addr<Motor>;
    /// Returns the address of the motor associated with the given buffer.
    fn index(&self, i: MotorId) -> &Self::Output {
        &self.motors[i]
    }
}

/// Stores motors and pump until it's time to start them.
#[derive(Debug)]
struct Devices {
    motors: Vec<Motor>,
    pump: Pump,
}

/// Contains program and buffer states.
#[derive(Debug, Default)]
pub(crate) struct CoordState {
    /// The currently-in-progress (original) program.
    pub(crate) program: Option<Program>,
    /// The steps remaining, derived from the original program.
    pub(crate) remaining: Vec<Action>,
    /// The step we're currently running.
    pub(crate) current: Option<Action>,
    /// The most recent buffer.
    pub(crate) buffer: Option<MotorId>,
    /// The current status of program execution.
    pub(crate) status: State,
    /// The completed steps of the program.
    pub(crate) completed: Vec<Action>,
    /// The uuid associated with the running (or most recently-completed) job.
    pub(crate) uuid: Option<Uuid>,
}

/// Contains all the actual logic for controlling the system based on a specified program.
#[derive(Debug)]
pub struct Coordinator {
    /// The devices this coordinator controls.
    ///
    /// Once the coordinator is started, this will be `None`.
    devices: Option<Devices>,
    /// The handles giving us access to everything.
    ///
    /// This will be `None` until the coordinator is started.
    // TODO: Express this better (and maybe start unwrapping)
    addresses: Option<Addresses>,
    /// Encodes the state of the coordinator.
    pub(crate) state: CoordState,
    /// The contact emails of the administrators of this machine.
    admins: Vec<String>,
}

impl Coordinator {
    /// Initializes a coordinator and prepares it for running.
    pub fn try_new(config: Config) -> Result<Self> {
        let mut pump = Pump::try_new(config.pump.pins)?;
        pump.invert = config.pump.invert;
        let motors = config
            .motors
            .into_iter()
            .map(|spec| {
                // TODO: Implement labels
                let period = spec.period;
                let range = spec.range[0]..=spec.range[1];
                let pin = spec.pin;
                Motor::try_new(period, range, pin)
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let devices = Some(Devices { motors, pump });
        Ok(Self {
            devices,
            addresses: None,
            state: CoordState::default(),
            admins: config.admins,
        })
    }
    /// The in-progress program, if appropriate.
    pub fn program(&self) -> Option<&Program> {
        self.state.program.as_ref()
    }
    /// The current status of the coordinator.
    pub fn status(&self) -> State {
        self.state.status
    }
    /// Closes all valves, shutting the waste valve.
    fn close_all(&self, context: &mut CoordContext) {
        if let Some(ref addresses) = self.addresses {
            addresses[0].do_send(MotorMessage::Shut);
            for addr in addresses.motors.iter().skip(1) {
                addr.do_send(MotorMessage::Close);
            }
        }
        context.run_later(Duration::new(5, 0), move |coord, _| {
            if let Some(ref addresses) = coord.addresses {
                for addr in &addresses.motors {
                    addr.do_send(MotorMessage::Stop);
                }
            }
        });
    }
    fn _close(&self, index: usize, context: &mut CoordContext) {
        if let Some(ref addresses) = self.addresses {
            addresses[index].do_send(MotorMessage::Close);
            context.run_later(Duration::new(5, 0), move |coord, _| {
                if let Some(ref addresses) = coord.addresses {
                    addresses[index].do_send(MotorMessage::Stop);
                }
            });
        }
    }
    fn close(&self, valve: usize, context: &mut CoordContext) {
        let index = valve + 1; // Valve 0 is waste
        self._close(index, context);
    }
    fn _open(&self, index: usize, context: &mut CoordContext) {
        if let Some(ref addresses) = self.addresses {
            addresses[index].do_send(MotorMessage::Open);
            context.run_later(Duration::new(5, 0), move |coord, _| {
                if let Some(ref addresses) = coord.addresses {
                    addresses[index].do_send(MotorMessage::Stop);
                }
            });
        }
    }
    fn open(&self, valve: usize, context: &mut CoordContext) {
        let index = valve + 1; // Valve 0 is waste
        self._open(index, context);
    }
    fn shut_waste(&self, context: &mut CoordContext) {
        if let Some(ref addresses) = self.addresses {
            addresses[0].do_send(MotorMessage::Shut);
            context.run_later(Duration::new(5, 0), move |coord, _| {
                if let Some(ref addresses) = coord.addresses {
                    addresses[0].do_send(MotorMessage::Stop);
                }
            });
        }
    }
    fn open_waste(&self, context: &mut CoordContext) {
        self._open(0, context);
    }
    fn close_waste(&self, context: &mut CoordContext) {
        self._close(0, context);
    }
    fn perfuse(&self) {
        if let Some(ref addresses) = self.addresses {
            addresses.pump.do_send(PumpMessage::Perfuse);
        }
    }
    fn drain(&self) {
        if let Some(ref addresses) = self.addresses {
            addresses.pump.do_send(PumpMessage::Drain);
        }
    }
    fn stop_pump(&self) {
        if let Some(ref addresses) = self.addresses {
            addresses.pump.do_send(PumpMessage::Stop);
        }
    }
    /// Attempts to run the next step of the program, aborting and cleaning up on failure.
    fn try_advance(&mut self, context: &mut CoordContext) {
        let result = self.advance(context);
        if let Err(err) = result {
            // TODO: Notify user
            log::error!("Aborting due to program advance error: {:?}", err);
            let mut tries = 0;
            let mut result = self.hcf();
            while tries < 5 && result.is_err() {
                std::thread::sleep(Duration::from_millis(200));
                result = self.hcf();
                tries += 1;
            }
            if result.is_err() {
                log::error!("Could not fully stop program; please take caution!");
            }
        }
    }
    /// Moves to the next step of the program, returning the new current action.
    fn advance(&mut self, context: &mut CoordContext) -> Result<Option<Action>> {
        if !self.state.remaining.is_empty() {
            self.state.status = State::Running;
            let action = self.state.remaining.remove(0);
            // Make sure to message something that will call advance again later!
            // Usually this will be try_advance.
            match action.clone() {
                Action::Perfuse(buffer) => {
                    self.shut_waste(context);
                    self.open(buffer, context);
                    self.perfuse();
                    context.run_later(*DURATION, move |coord, context| {
                        coord.close(buffer, context);
                        coord.open_waste(context);
                        // Clear the line for ten seconds
                        context.run_later(Duration::new(10, 0), move |coord, context| {
                            coord.stop_pump();
                            coord.close_waste(context);
                            coord.try_advance(context);
                        });
                    });
                }
                Action::Sleep(duration) => {
                    context.run_later(duration, Self::try_advance);
                }
                Action::Hail => {
                    self.state.status = State::Waiting;
                    // TODO: Publish for other actions as well
                    self.publish(StatusMessage::Paused, context);
                }
                Action::Drain => {
                    self.close_waste(context);
                    self.drain();
                    context.run_later(*DURATION + Duration::new(5, 0), |coord, context| {
                        coord.stop_pump();
                        coord.shut_waste(context);
                        coord.try_advance(context);
                    });
                }
                Action::Finish => {
                    self.stop_pump();
                    self.close_all(context);
                    // TODO: Handle error
                    let _ = mail::notify(&self.admins, mail::Status::Finished);
                    // TODO: Update coordinator state
                }
                Action::Notify(msg) => {
                    log::trace!("Notifying user (subject: {}).", msg.subject);
                    // TODO: Handle error
                    let _ = mail::mail(&self.admins, msg.subject, msg.message);
                    self.try_advance(context);
                }
            }
            self.state.completed.push(action.clone());
            self.state.current = Some(action);
        } else {
            self.state.status = State::Stopped { early: false };
            self.state.current = None;
        }
        Ok(self.state.current.clone())
    }
    /// Clears the remaining program queue after the next perfusion.
    fn clear(&mut self) -> Result<()> {
        if let Some(index) = self.state.remaining.iter().position(Action::is_disjoint) {
            // Vec::truncate keeps n elements, but we don't want to keep the element at index.
            self.state.remaining.truncate(index);
        }
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
            log::warn!("Coordinator told to resume while not paused; ignoring.");
            return Ok(());
        }
        self.state.status = State::Running;
        self.advance(context)?;
        Ok(())
    }
    /// Abort the program no matter where we are.
    fn hcf(&mut self) -> Result<()> {
        self.stop_pump();
        // TODO: Reset motors?
        self.state.status = State::Stopped { early: true };
        // We didn't finish the last step, so remove it from the list
        self.state.completed.pop();
        // TODO: Handle error
        let _ = mail::notify(&self.admins, mail::Status::Aborted);
        Ok(())
    }
    /// Whether we're in the stopped state.
    pub fn is_stopped(&self) -> bool {
        match self.state.status {
            State::Stopped { .. } => true,
            State::Running | State::Waiting => false,
        }
    }
    /// Start the given protocol, if we can.
    fn start(
        &mut self,
        protocol: &Protocol,
        label: Option<Uuid>,
        context: &mut CoordContext,
    ) -> Result<()> {
        let program = protocol.as_program()?;
        if self.is_stopped() {
            self.stop_pump();
            self.close_all(context);
            context.run_later(Duration::new(10, 0), move |coord, context| {
                let id = label.unwrap_or_else(Uuid::new_v4);
                coord.state.program = Some(program.clone());
                coord.state.remaining = program.into();
                coord.state.current = None;
                coord.state.buffer = None;
                coord.state.status = State::Running;
                coord.state.completed.clear();
                coord.state.uuid = Some(id);
                coord.advance(context).unwrap();
            });
        }
        Ok(())
    }
    /// Subscribes the given object to updates from the coordinator.
    pub fn subscribe(&self, sub: Box<dyn Update>) {
        if let Some(addr) = &self.addresses {
            addr.subscribers.do_send(SubscribersMessage::Add(sub));
        }
    }
    /// Publishes a status change to all subscribers.
    fn publish(&self, message: StatusMessage, context: &mut <Self as Actor>::Context) {
        if let Some(addr) = &self.addresses {
            let message = Status {
                address: context.address(),
                message,
            };
            addr.subscribers
                .do_send(SubscribersMessage::Forward(message));
        }
    }
}

impl Actor for Coordinator {
    type Context = CoordContext;
    fn started(&mut self, ctx: &mut Self::Context) {
        let subscribers = Subscribers {
            subs: vec![],
            coord: ctx.address(),
        }
        .start();
        if let Some(devices) = self.devices.take() {
            let motors = devices
                .motors
                .into_iter()
                .map(Actor::start)
                .collect::<Vec<_>>();
            let pump = devices.pump.start();
            let addresses = Addresses {
                pump,
                motors,
                subscribers,
            };
            self.addresses = Some(addresses);
        }
    }
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        // Redundant due to the impending drop, but I like to be explicit
        self.addresses = None;
    }
}

impl Handle<Message> for Coordinator {
    type Result = Result<()>;
    fn handle(&mut self, message: Message, context: &mut Self::Context) -> Self::Result {
        match message {
            Message::Continue => {
                self.resume(context)?;
                self.publish(StatusMessage::Continued, context);
            }
            Message::Stop => {
                self.stop(None)?;
                self.publish(StatusMessage::StopQueued { early: false }, context);
            }
            Message::Halt => {
                self.hcf()?;
                self.publish(StatusMessage::Halted, context);
            }
            Message::ExchangeStop(id) => {
                self.stop(id)?;
                self.publish(StatusMessage::StopQueued { early: false }, context);
            }
            Message::Start(proto, label) => {
                self.start(&proto, label, context)?;
                self.publish(StatusMessage::Started(proto), context);
            }
            Message::Subscribe(sub) => self.subscribe(sub),
        }
        Ok(())
    }
}

#[derive(Debug)]
enum SubscribersMessage {
    /// Register a new listener.
    Add(Box<dyn Update>),
    /// Forward this message to listeners.
    Forward(Status),
}

impl ActixMessage for SubscribersMessage {
    type Result = ();
}

/// Handles all subscription and responding to events.
#[derive(Debug)]
pub struct Subscribers {
    coord: Addr<Coordinator>,
    subs: Vec<Box<dyn Update>>,
}

impl Actor for Subscribers {
    type Context = Context<Self>;
}

impl Handle<SubscribersMessage> for Subscribers {
    type Result = ();
    fn handle(&mut self, message: SubscribersMessage, _context: &mut Self::Context) {
        match message {
            SubscribersMessage::Forward(message) => {
                for sub in self.subs.iter() {
                    sub.handle(&message, &self);
                }
            }
            SubscribersMessage::Add(listener) => {
                self.subs.push(listener);
            }
        }
    }
}

pub trait Respond {
    fn respond(&self, msg: Message);
}

impl Respond for Subscribers {
    fn respond(&self, msg: Message) {
        self.coord.do_send(msg);
    }
}

/// Trait for receiving updates on coordinator status.
pub trait Update: std::fmt::Debug + Send {
    /// Handles the change in coordinator status.
    fn handle(&self, msg: &Status, coord: &Subscribers);
}

#[derive(Debug)]
/// Message notifying subscribers of changes in the coordinator's status.
pub struct Status {
    /// The address of the coordinator in question.
    pub address: Addr<Coordinator>,
    /// The information the coordinator wishes to convey.
    pub message: StatusMessage,
}

#[derive(Debug)]
/// Encodes a coordinator's status update.
pub enum StatusMessage {
    /// The coordinator has been told to continue.
    Continued,
    /// The coordinator has started the given protocol.
    Started(Protocol),
    /// The coordinator has paused and will await user confirmation to continue.
    Paused,
    /// The coordinator has been told to stop, either early (aborted) or not (completed).
    StopQueued {
        /// Whether the stop was premature.
        early: bool,
    },
    /// The coordinator has been halted.
    Halted,
}

impl ActixMessage for Status {
    type Result = ();
}

#[allow(clippy::print_stdout)]
pub mod tui {
    use super::{Message, Respond, Status, StatusMessage, Subscribers, Update};
    /// A helper which allows the user to continue the coordinator by sending a newline.
    // Don't impl Clone or Copy; we don't want multiple responders of this type.
    #[allow(missing_copy_implementations)]
    #[derive(Debug, Default)]
    pub struct Tui {}
    impl Update for Tui {
        fn handle(&self, status: &Status, coord: &Subscribers) {
            match &status.message {
                StatusMessage::Paused => {
                    log::trace!("Prompting user to unpause.");
                    use std::io::{stdin, stdout, BufRead, BufReader, Write};
                    let stdin = stdin();
                    let mut stdin = BufReader::new(stdin.lock());
                    print!("Coordinator paused. Press enter to continue when desired.");
                    let _ = stdout().lock().flush();
                    let mut s = String::new();
                    loop {
                        if stdin.read_line(&mut s).is_ok() {
                            break;
                        }
                    }
                    coord.respond(Message::Continue);
                }
                StatusMessage::Continued => log::debug!("Coordinator continuing."),
                StatusMessage::Started(proto) => {
                    log::debug!("Coordinator starting protocol: {:?}", proto)
                }
                StatusMessage::StopQueued { early } => {
                    log::debug!("Coordinator stop queued (early: {})", early)
                }
                StatusMessage::Halted => log::warn!("Coordinator halted!"),
            }
        }
    }

}
