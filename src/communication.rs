use motion::Motor;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;
use std::ops::Range;

pub enum Action {
	Stop,
	SetOrthogonal,
	SetParallel
}

pub struct Slave {
	motor: Arc<Mutex<Motor>>,
	rx: mpsc::Receiver<Action>
}

impl Slave {

	pub fn create_with_channel(pin_number: u16, period: Duration, signal_range: Range<Duration>) -> (Self, mpsc::Sender<Action>) {
		let (tx, rx) = mpsc::channel();
		let motor = Arc::new(Mutex::new(Motor::new(pin_number, period, signal_range)));
		(Self {
			rx,
			motor
		}, tx)
	}

	/// Handles all messages sent to the thread.
	fn handle(&mut self, message: Action) {
		match message { // TODO: Error handling
			Action::Stop => { self.motor.lock().unwrap().set_neutral(); }, // TODO: Clear queue
			Action::SetOrthogonal => { self.motor.lock().unwrap().set_orthogonal(); },
			Action::SetParallel => { self.motor.lock().unwrap().set_neutral(); },
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
		while let Ok(action) = self.rx.recv() {
			self.handle(action);
		}
	}

}
