use config::MotorSpec;

use motion::Motor;
use std::collections::VecDeque;
use std::ops::Range;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use angle::Angle;
pub type Delay = Duration;

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

pub type ScheduledAction = (Instant, Action);

pub struct Slave {
	motor: Arc<Mutex<Motor>>,
	rx: mpsc::Receiver<Action>,
	queue: VecDeque<ScheduledAction>
}

impl Slave {
	/// Creates a slave and communication (mpsc) channel for the given motor specs.
    pub fn create_with_channel(
        pin_number: u16,
        period: Duration,
        signal_range: Range<Duration>,
    ) -> (Self, mpsc::Sender<Action>) {
        let (tx, rx) = mpsc::channel();
        let motor = Arc::new(Mutex::new(Motor::new(pin_number, period, signal_range)));
        (
            Self {
                rx,
                motor,
                queue: VecDeque::new(),
            },
            tx,
        )
    }

    /// Handles all messages sent to the thread.
    fn handle(&mut self, message: Action) {
        match message {
            // TODO: Error handling
            Action::Stop => {
                self.motor.lock().unwrap().set_neutral();
                println!("Set motor neutral at instant {:?}", Instant::now());
            } // TODO: Clear queue
            Action::Close => {
                self.motor.lock().unwrap().set_neutral();
                println!("Set motor neutral at instant {:?}", Instant::now());
            }
            Action::Open(length) => {
                self.motor.lock().unwrap().set_orthogonal();
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
                self.motor.lock().unwrap().set_angle(angle).unwrap();
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
        let motor = Arc::clone(&self.motor);
        let _child = thread::spawn(move || loop {
            let mut motor = motor.lock().unwrap();
            motor.do_wave();
        });
        loop {
            match self.rx.try_recv() {
                Ok(action) => self.handle(action),
                _ => {} // TODO: Handle error
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

pub struct Coordinator {
	channels: Vec<mpsc::Sender<Action>>
}

impl From<Vec<MotorSpec>> for Coordinator {

impl Drop for Coordinator {
	fn drop(&mut self) {
		while let Some(channel) = self.channels.pop() {
			channel.send(Action::Close).unwrap();
		}
	}
}
