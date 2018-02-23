use motion::Motor;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};
use std::ops::Range;
use std::collections::VecDeque;

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
	ScheduleClose(Delay)
}

pub type ScheduledAction = (Instant, Action);

pub struct Slave {
	motor: Arc<Mutex<Motor>>,
	rx: mpsc::Receiver<Action>,
	queue: VecDeque<ScheduledAction>
}

impl Slave {

	pub fn create_with_channel(pin_number: u16, period: Duration, signal_range: Range<Duration>) -> (Self, mpsc::Sender<Action>) {
		let (tx, rx) = mpsc::channel();
		let motor = Arc::new(Mutex::new(Motor::new(pin_number, period, signal_range)));
		(Self {
			rx,
			motor,
			queue: VecDeque::new()
		}, tx)
	}

	/// Handles all messages sent to the thread.
	fn handle(&mut self, message: Action) {
		match message { // TODO: Error handling
			Action::Stop => { self.motor.lock().unwrap().set_neutral(); }, // TODO: Clear queue
			Action::Close => { self.motor.lock().unwrap().set_neutral(); },
			Action::Open(length) => {
				self.motor.lock().unwrap().set_orthogonal();
				self.handle(Action::ScheduleClose(length));
			},
			Action::ScheduleOpen(delay, length) => {
				self.queue.push_back((Instant::now() + delay, Action::Open(length)));
			},
			Action::ScheduleClose(delay) => {
				self.queue.push_back((Instant::now() + delay, Action::Close));
			}
		}
	}

	/// The entire *raison d'être* for `Slave` instances.
	/// This method causes both the motor and the handler to loop.
	/// # Notes
	/// You **must** call this method for this struct to be useful at all.
	pub fn _loop(mut self) { // TODO: Handle timing as well (instead of performing everything on the next tick)
		let motor = self.motor.clone();
		thread::spawn(move ||{
			loop {
				let mut motor = motor.lock().unwrap();
				motor.do_wave();
			}
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
