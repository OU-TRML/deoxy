use std::sync::mpsc;
use std::time::Duration;
use std::thread;
extern crate gpio;
pub mod io;
pub mod motion;

#[allow(unused_imports)]
use io::{Pin, GpioOutputStub};
use motion::Motor;

fn main() {
	let (tx, rx) = mpsc::channel();
	let child = thread::spawn(move || {
		let pin = rx.recv().unwrap();
		let mut motor = Motor::new(pin, Duration::from_millis(20), Duration::new(0, 553_000)..Duration::new(0, 2_520_000));
		let total = Duration::from_millis(20);
		let _ = motor.set_angle(0.0).unwrap();
		let zero = motor.get_pulse_width();
		motor.set_neutral();
		let neutral = motor.get_pulse_width();
		for _ in 0..10 {
			for _ in 0..20 {
				motor.do_wave(neutral, total);
			}
			for _ in 0..20 {
				motor.do_wave(zero, total);
			}
		}
	});
	let _ = tx.send(17);
	let result = child.join();
	if let Err(err) = result {
		println!("Error: {:?}", err);
	}
}
