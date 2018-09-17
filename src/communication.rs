//! Contains components essential for communication between threads and to motors, etc.

use config::MotorSpec;

use motion::{Motor, MotorRange};
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
#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub enum Action {
    /// Stops everything that's going on, clears the queue, and closes the tube.
    Stop,
    /// Opens the tube for the specified duration (approximately).
    Open(Duration),
    /// Closes the tube. Unlike `Stop`, `Close` does not clear the queue.
    Close,
    /// Schedules an open event for later.
    ScheduleOpen(Delay, Duration),
    /// Schedules a close event for later.
    ScheduleClose(Delay),
    /// Sets the motor to a custom angle for the specified duration.
    SetAngle(Angle, Duration),
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
}

impl Slave {
    /// Creates a slave and communication (mpsc) channel for the given motor specs.
    pub fn create_with_channel(spec: MotorSpec) -> (Self, mpsc::Sender<Action>) {
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
            }
            Action::Close => {
                self.set_neutral();
                println!("Set motor neutral at instant {:?}", Instant::now());
            }
            Action::Open(length) => {
                self.set_orthogonal();
                println!("Set motor orthogonal at instant {:?}", Instant::now());
                self.handle(Action::ScheduleClose(length));
            }
            Action::ScheduleOpen(delay, length) => {
                self.queue
                    .push_back((Instant::now() + delay, Action::Open(length)));
            }
            Action::ScheduleClose(delay) => {
                self.queue
                    .push_back((Instant::now() + delay, Action::Close));
            }
            Action::SetAngle(angle, length) => {
                self.set_angle(angle).unwrap();
                println!(
                    "Set motor angle to {} at instant {:?}",
                    angle,
                    Instant::now()
                );
                self.handle(Action::ScheduleClose(length));
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
                self.handle(action);
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
/// *will* all close.
#[derive(Debug)]
pub struct Coordinator {
    /// The communication channels to
    pub channels: Vec<mpsc::Sender<Action>>,
}

impl<'a> From<&'a [MotorSpec]> for Coordinator {
    ///
    ///
    /// #Notes
    /// This method will spin up child threads and start `Slave`s looping (which is why it moves its
    /// argument). Once it has been used, the motors *will* be receiving messages immediately.
    ///
    /// The message sent to the motor is simply `Action::Close`, so that the motor immediately goes
    /// to the closed position if it is not already.
    /// #Panics
    /// This method will `panic!` if sending the `Action::Close` message to the child thread fails.
    fn from(motors: &'a [MotorSpec]) -> Self {
        Self {
            channels: motors
                .iter()
                .map(|spec| Slave::create_with_channel(spec.clone()))
                .map(move |(slave, maw)| {
                    let _child = thread::spawn(move || {
                        slave._loop();
                    });
                    maw.send(Action::Close).unwrap(); // TODO: Error handling
                    maw
                })
                .collect(),
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
