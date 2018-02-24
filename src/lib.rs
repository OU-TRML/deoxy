use std::time::Duration;
use std::thread;

extern crate gpio;
#[macro_use]
extern crate serde_derive;
extern crate toml;

pub mod io;
pub mod motion;
pub mod communication;
pub mod config;

#[allow(unused_imports)]
use io::{Pin, GpioOutputStub};
use communication::{Slave, Action};
use config::Config;

pub fn main(config: Config) {
	let first = config.motors.first.unwrap();
	let pin = first.pin;
	let (period, min, max) = (Duration::from_millis(first.period), Duration::new(0, first.min * 1000), Duration::new(0, first.max * 1000));
	let (slave, maw) = Slave::create_with_channel(pin, period, min..max);
	let child = thread::spawn(move || {
		slave._loop();
	});
	let _ = maw.send(Action::Close).unwrap(); // TODO: Error handling
	let _ = maw.send(Action::Open(Duration::from_millis(first.period * 100))).unwrap(); // TODO: Error handling
	let result = child.join();
	if let Err(err) = result {
		println!("Error: {:?}", err);
	}
}
