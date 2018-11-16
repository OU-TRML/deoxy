//! Contains components essential for communication between threads and to motors, etc.

use config::{Config, MotorSpec};

use motion::{Motor, MotorRange, Pump};
use std::collections::VecDeque;
use std::ops::Range;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use angle::Angle;

/// Represents a delay to rest before taking (or completing) an action.
pub type Delay = Duration;

/// Encodes an action that a motor can take.
// We don't want duplicate actions, so we disable Copy and make move semantics matter for this.
///
/// #### Notes
///
/// In order to handle the case of filling and waiting for the user to do something, the
/// currently-supported method is to request an `Open(None)` at time `t`, followed by an
/// appropriately-scheduled (`t + length`) `Close(None)` (which could also be scheduled with
/// `ScheduleClose`), followed immediately by a `Block`, followed immediately by a
/// `Close(length)`. This is obviously not very fun to work with, so we'll probably provide a
/// more convenient method later (see [#18](https://github.com/Aehmlo/deoxy/issues/18)).
#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub enum Action {
    /// Stops everything that's going on, clears the queue, and closes the tube.
    Stop,
    /// Opens the tube for the specified duration (approximately), or indefinitely if None.
    Open(Option<Duration>),
    /// Closes the tube. Unlike `Stop`, `Close` does not clear the queue.
    ///
    /// If a duration is given, it is the amount of time for which to allow the pump to drain.
    /// In this case, another close with no duration is scheduled for immediately after the pump is
    /// done to clean up (free the pump).
    Close(Option<Duration>),
    /// Schedules an open event for later.
    ScheduleOpen(Delay, Option<Duration>),
    /// Schedules a close event for later.
    ///
    /// If a duration is also given, it is the amount of time for which to allow the pump to drain.
    ScheduleClose(Delay, Option<Duration>),
    /// Sets the motor to a custom angle for the specified duration.
    SetAngle(Angle, Duration),
    /// Blocks the slave until asked to unblock.
    Block,
    /// Indicates that we're blocking.
    ///
    /// This variant stores the instant when blocking started so that we can adjust later.
    KeepBlocking(Instant),
    /// Unblocks the receiver, allowing the process flow to continue.
    Unblock,
}

/// An future action that is expected to happen.
///
/// The `Instant` associated with this pair is when the action is expected to occur.
pub type ScheduledAction = (Instant, Action);

/// Manages actions and messaging (including threading) for a single motor.
#[derive(Debug)]
pub struct Slave {
    rx: mpsc::Receiver<Action>,
    queue: VecDeque<ScheduledAction>,
    pulse_width: Option<Arc<Mutex<Duration>>>,
    signal_range: Range<Duration>,
    period: Duration,
    pin: u16,
    pump_ref: Arc<Mutex<Pump>>,
    pump_in_use: Arc<Mutex<bool>>,
    /// Shared state indicating whether the Slave is blocking.
    ///
    /// Used to tell the user their help is needed.
    pub is_blocking: Arc<Mutex<bool>>,
}

impl Slave {
    /// Creates a slave and communication (mpsc) channel for the given motor specs.
    pub fn slave_and_channel(
        spec: MotorSpec,
        pump_ref: Arc<Mutex<Pump>>,
        pump_in_use: Arc<Mutex<bool>>,
    ) -> (Self, mpsc::Sender<Action>) {
        let pin = spec.get_pin();
        let (period, min, max) = (
            Duration::from_millis(spec.get_period()),
            Duration::new(0, spec.get_min() * 1000),
            Duration::new(0, spec.get_max() * 1000),
        );
        let (tx, rx) = mpsc::channel();
        (
            Self {
                rx,
                queue: VecDeque::new(),
                pulse_width: None,
                signal_range: min..max,
                period,
                pin,
                pump_ref,
                pump_in_use,
                is_blocking: Arc::new(Mutex::new(false)),
            },
            tx,
        )
    }

    /// Sets the motor to the neutral position.
    pub fn set_neutral(&mut self) {
        let pulse_width = self
            .pulse_width
            .clone()
            .expect("Motor moved while not looping.");
        let mut pulse_width = pulse_width.lock().unwrap();
        *pulse_width = (self.signal_range.start + self.signal_range.end) / 2;
    }

    /// Sets the motor angle.
    /// # Errors
    /// If the given `angle` doesn't lie within `angle_range`, this method returns Err(()) and nothing happens.
    pub fn set_angle(&mut self, angle: Angle) -> Result<(), ()> {
        // TODO: Allow configuration.
        let angle_range = MotorRange::default().to_range();
        if angle < angle_range.start || angle > angle_range.end {
            Err(())
        } else {
            let ratio = (self.signal_range.end - self.signal_range.start)
                / ((angle_range.end - angle_range.start).measure() as u32);
            let (seconds, nanoseconds) = (ratio.as_secs(), ratio.subsec_nanos());
            let pulse_width = self
                .pulse_width
                .clone()
                .expect("Motor moved while not looping.");
            let mut pulse_width = pulse_width.lock().unwrap();
            *pulse_width = Duration::new(
                ((seconds as f64) * angle.measure()).round() as u64,
                (f64::from(nanoseconds) * angle.measure()).round() as u32,
            );
            Ok(())
        }
    }

    /// Sets the motor to the "zero" position.
    pub fn set_orthogonal(&mut self) {
        self.set_angle(Angle::with_measure(0.0)).unwrap();
    }

    /// Pushes back all scheduled actions by a specified duration.
    fn delay_steps(&mut self, delta: Duration) {
        for action in self.queue.iter_mut() {
            action.0 = action.0 + delta;
        }
    }

    /// Handles all messages sent to the thread.
    #[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
    fn handle(&mut self, message: Action) {
        match message {
            // TODO: Error handling
            Action::Stop => {
                self.set_neutral();
                println!("Set motor neutral at instant {:?}", Instant::now());
                {
                    let _queue = self.queue.drain(..);
                }
                assert_eq!(self.queue.len(), 0);
                let _we_tried = self.pump_ref.try_lock().map(|mut p| p.close());
            }
            Action::Close(drain_time) => {
                self.set_neutral();
                println!("Set motor neutral at instant {:?}", Instant::now());
                if let Some(time) = drain_time {
                    let start = Instant::now();
                    let r = self.pump_ref.clone();
                    let mut pump = r.lock().unwrap();
                    pump.drain();
                    let delta = Instant::now() - start;
                    self.delay_steps(delta);
                    self.handle(Action::ScheduleClose(time, None));
                } else {
                    let start = Instant::now();
                    self.pump_ref.lock().unwrap().close();
                    *self.pump_in_use.lock().unwrap() = false;
                    let delta = Instant::now() - start;
                    self.delay_steps(delta);
                }
            }
            Action::Open(length) => {
                let start = Instant::now();
                // Block until we can get the pump.
                loop {
                    let mut pump_in_use = self.pump_in_use.lock().unwrap();
                    if !*pump_in_use {
                        *pump_in_use = true;
                        break;
                    }
                }
                let r = self.pump_ref.clone();
                let mut pump = r.lock().unwrap();
                // Now, get the current instant and use it to perform correction on all the scheduled actions.
                let now = Instant::now();
                let delta = now - start;
                self.delay_steps(delta);
                // Open the motor and proceed
                println!("Set motor orthogonal at instant {:?}", now);
                self.set_orthogonal();
                pump.perfuse();
                if let Some(l) = length {
                    self.handle(Action::ScheduleClose(l, Some(l)));
                } else {
                    *self.pump_in_use.lock().unwrap() = false;
                }
            }
            Action::ScheduleOpen(delay, length) => {
                self.queue
                    .push_back((Instant::now() + delay, Action::Open(length)));
            }
            Action::ScheduleClose(delay, length) => {
                self.queue
                    .push_back((Instant::now() + delay, Action::Close(length)));
            }
            Action::SetAngle(angle, length) => {
                self.set_angle(angle).unwrap();
                println!(
                    "Set motor angle to {} at instant {:?}",
                    angle,
                    Instant::now()
                );
                self.handle(Action::ScheduleClose(length, Some(length)));
            }
            Action::Block => {
                *self.is_blocking.lock().unwrap() = true;
                // TODO(#17): Notify user that we're blocking.
                let now = Instant::now();
                self.queue.push_front((now, Action::KeepBlocking(now)));
            }
            Action::KeepBlocking(since) => {
                self.queue.push_front((since, Action::KeepBlocking(since)));
            }
            Action::Unblock => {
                panic!("Encountered Action::Unblock in Slave queue.");
            }
        }
    }

    /// The entire *raison d'être* for `Slave` instances.
    /// This method causes both the motor and the handler to loop.
    /// # Notes
    /// You **must** call this method for this struct to be useful at all.
    pub fn _loop(mut self) {
        // TODO: Handle timing as well (instead of performing everything on the next tick)
        let mut motor = Motor::new(self.pin, self.period);
        self.pulse_width = Some(motor.get_pulse_width());
        let _child = thread::spawn(move || loop {
            motor.do_wave();
        });
        loop {
            if let Ok(action) = self.rx.try_recv() {
                match action {
                    Action::Unblock => {
                        // This could go in the match, but we'll be safe and put it here.
                        *self.is_blocking.lock().unwrap() = false;
                        match self.queue.pop_front() {
                            Some((_, Action::KeepBlocking(start))) => {
                                let now = Instant::now();
                                let delta = now - start;
                                self.delay_steps(delta);
                            }
                            Some(a) => self.queue.push_front(a),
                            None => {}
                        }
                    }
                    a => self.handle(a),
                }
            }
            if let Some(action) = self.queue.pop_front() {
                let now = Instant::now();
                if now >= action.0 {
                    self.handle(action.1);
                } else {
                    self.queue.push_front(action);
                }
            }
        }
    }
}

/// The entry point for all messages sent to motors.
///
/// This struct owns all the communication channels. Don't let it go out of scope, or the motors
/// *will* all close (and so will the pump).
#[derive(Debug)]
pub struct Coordinator {
    /// The communication channels to
    pub channels: Vec<mpsc::Sender<Action>>,
    /// The pump.
    pub pump: Arc<Mutex<Pump>>,
    /// This pump_in_use is false if the pump has been claimed by a slave.
    pub pump_is_free: Arc<Mutex<bool>>,
}

impl<'a> From<&'a Config> for Coordinator {
    ///
    ///
    /// #Notes
    /// This method will spin up child threads and start `Slave`s looping. Once it has been used,
    /// the motors *will* be receiving messages immediately.
    ///
    /// The message sent to the motor is simply `Action::Close`, so that the motor immediately goes
    /// to the closed position if it is not already.
    /// #Panics
    /// This method will `panic!` if sending the `Action::Close` message to the child thread fails.
    fn from(config: &'a Config) -> Self {
        let motors = config.motors();
        let pump_is_free = Arc::new(Mutex::new(true));
        let pump = Arc::new(Mutex::new(config.pump().into()));
        Self {
            channels: motors
                .iter()
                .map(|spec| {
                    Slave::slave_and_channel(spec.clone(), pump.clone(), pump_is_free.clone())
                })
                .map(move |(slave, maw)| {
                    let _child = thread::spawn(move || {
                        slave._loop();
                    });
                    maw.send(Action::Close(None)).unwrap(); // TODO: Error handling
                    maw
                })
                .collect(),
            pump,
            pump_is_free,
        }
    }
}

impl Drop for Coordinator {
    /// When a Coordinator goes out of scope, it sends `Action::Stop` through all of its channels,
    /// closing all registered motors and clearing their queues.
    fn drop(&mut self) {
        while let Some(channel) = self.channels.pop() {
            channel.send(Action::Stop).unwrap();
        }
    }
}
