use std::thread;
use std::sync::mpsc;
use std::time::Duration;

mod motor;
use motor::Motor;

pub enum Message {
	Debug(String),
	SetPulseWidthForDuration(Duration, Duration),
	SetPulseWidth(Duration),
	Stop
}

pub struct Hub {
	senders: Vec<mpsc::Sender<Message>>
}

impl Hub {

	pub fn new(order: u8) -> Self {
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

	pub fn send(&self, to: usize, packet: Message) -> Result<(), mpsc::SendError<Message>> {
		self.senders[to].send(packet)
	}

}

struct Slave {
	receiver: mpsc::Receiver<Message>,
	id: u8,
	motor: Motor
}

impl Slave {

	fn new(id: u8, receiver: mpsc::Receiver<Message>, pin: u8) -> Self {
		Self {
			receiver,
			id,
			motor: Motor::new(pin, Duration::from_millis(20), (Duration::new(0, 900_000), Duration::new(0, 1_200_000)))
		}
	}

	fn _loop(&mut self) {
		while let Ok(message) = self.receiver.recv() {
			match message {
				Message::Debug(m) => println!("Thread {} received message \"{}\"", self.id, m),
				Message::SetPulseWidth(width) => { let _ = self.motor.set_pulse_width(width); }, // TODO: Error handling
				Message::Stop => { let _ = self.motor.set_neutral(); },
				_ => {}
			};
		}
	}

}
