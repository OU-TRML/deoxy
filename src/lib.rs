mod motor;
pub mod config;

pub use motor::{Motor, MotorError, MotorRange};

/// All things related to motor control and management lie within this module.
pub mod communication {

	use std::thread;
	use std::sync::mpsc;
	use std::time::Duration;
	use motor::Motor;
	use config::Config;

	/// Messages are used to effect changes in motor behavior, and are passed to appropriate child threads through a `Hub`.
	pub enum Message {
		/// Causes the receiving thread to print the given debug message.
		Debug(String),
		/// Sets the motor corresponding to the receiving thread to the given pulse width for a certain duration.
		SetPulseWidthForDuration(Duration, Duration),
		/// Sets the motor corresponding to the receiving thread to the given pulse width indefinitely.
		SetPulseWidth(Duration),
		/// Sets the motor corresponding to the receiving thread to the neutral position.
		Stop
	}

	/// The central dispatch location for sending all inter-thread messages (to control motors).
	///
	/// There should be one per application (though this is not enforced anywhere).
	pub struct Hub {
		senders: Vec<mpsc::Sender<Message>>
	}

	impl Hub {

		/// Constructs a new `Hub` with the given configuration.
		pub fn new(config: Config) -> Self {
			let mut senders = Vec::with_capacity(config.order);
			let mut i = 0;
			for spec in config.motors {
				let (tx, rx) = mpsc::channel();
				thread::spawn(move || {
					let mut slave = Slave::new(i, rx, spec.pin);
					slave._loop();
				});
				senders.push(tx);
				i += 1;
			}
			Self { senders }
		}

		/// Constructs a new `Hub` with the given number of threads.
		pub fn with_threads(order: u8) -> Self {
			let mut senders = Vec::with_capacity(order as usize);
			for i in 0..order {
				let (tx, rx) = mpsc::channel();
				thread::spawn(move|| {
					let mut slave = Slave::new(i, rx, i); // TODO: Replace last `i` with pin/motor
					slave._loop();
				});
				senders.push(tx);
			}
			Self {
				senders
			}
		}

		/// Attempts to send the given message to the thread at the given index (threads are, of course, zero-indexed).
		pub fn send(&self, to: usize, message: Message) -> Result<(), mpsc::SendError<Message>> {
			self.senders[to].send(message)
		}

	}

	/// Handles a single motor on a background thread, listening for messages from the main thread before acting (in event-loop-with-interupts style).
	struct Slave {
		receiver: mpsc::Receiver<Message>,
		id: u8,
		motor: Motor
	}

	impl Slave {

		/// Creates a new slave with the given ID which will listen using the passed receiver and move the motor on the given pin appropriately for the received messages.
		fn new(id: u8, receiver: mpsc::Receiver<Message>, pin: u8) -> Self {
			Self {
				receiver,
				id,
				motor: Motor::new(pin, Duration::from_millis(20), (Duration::new(0, 900_000), Duration::new(0, 1_200_000)))
			}
		}

		/// The main loop, wherein all logic happens on child (motor) threads.
		fn _loop(&mut self) {
			self.motor._loop();
			while let Ok(message) = self.receiver.recv() {
				let result = match message {
					Message::Debug(m) => { println!("Thread {} received message: \"{}\"", self.id, m); Ok(()) },
					Message::SetPulseWidth(width) => self.motor.set_pulse_width(width),
					Message::Stop => self.motor.set_neutral(), // TODO: Invalidate queue
					Message::SetPulseWidthForDuration(width, duration) => {
						let result = self.motor.set_pulse_width(width);
						self.motor.add_pulses(((duration.as_secs() as u64 * 1_000_000_000u64 / width.subsec_nanos() as u64)/1_000_000_000u64) as u32);
						result
					}
					_ => unimplemented!()
				};
			}
		}

	}
}
