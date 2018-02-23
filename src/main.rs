use std::time::Duration;
use std::thread;
extern crate gpio;
pub mod io;
pub mod motion;
pub mod communication;

#[allow(unused_imports)]
use io::{Pin, GpioOutputStub};
use communication::{Slave, Action};

fn main() {
	let pin = 17;
	let (period, min, max) = (Duration::from_millis(20), Duration::new(0, 553_000), Duration::new(0, 2_520_000)); // TODO: User input
	let (slave, maw) = Slave::create_with_channel(pin, period, min..max);
	let child = thread::spawn(move || {
		slave._loop();
	});
	let _ = maw.send(Action::Close).unwrap(); // TODO: Error handling
	let _ = maw.send(Action::Open(Duration::from_millis(2000))).unwrap(); // TODO: Error handling
	let result = child.join();
	if let Err(err) = result {
		println!("Error: {:?}", err);
	}
}
