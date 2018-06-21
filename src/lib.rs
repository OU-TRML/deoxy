//! A buffer-exchange crate.

#![warn(
    missing_copy_implementations, missing_debug_implementations, trivial_casts,
    trivial_numeric_casts, unused_extern_crates, unused_import_braces, unused_qualifications,
    unused_results
)]
#![deny(missing_docs)]

use std::thread;
use std::time::Duration;

extern crate gpio;
#[macro_use]
extern crate serde_derive;
extern crate toml;

mod angle;
pub mod communication;
pub mod config;
pub mod io;
pub mod motion;

use angle::Angle;
use communication::{Action, Coordinator};
use config::Config;
#[allow(unused_imports)]
use io::{GpioOutputStub, Pin};

/// Exactly what it says on the tin (for now).
#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
pub fn main(config: Config) {
    let mgr = Coordinator::from(config.motors());
    // mgr.channels[0].send(Action::Close).unwrap();
    mgr.channels[0]
        .send(Action::SetAngle(
            Angle::with_measure(180.0),
            Duration::from_millis(10_000),
        ))
        .unwrap();
    thread::sleep(Duration::from_millis(15_000));
    mgr.channels[0]
        .send(Action::SetAngle(
            Angle::with_measure(0.0),
            Duration::from_millis(10_000),
        ))
        .unwrap();
    thread::sleep(Duration::from_millis(15_000));
    // mgr.channels[0].send(Action::ScheduleOpen(Duration::from_millis(500), Duration::from_millis(2_000))).unwrap();
}

/*pub fn main(_config: Config) {
	let p = Duration::from_millis(20);
	let l = Duration::new(0, 553_000);
	let h = Duration::new(0, 2_520_000);

	let n = Duration::new(0, 1_536_000);

	for pin in [17, 27].iter() {
		let _ = thread::spawn(move || {
			let mut pin = Pin::new(*pin);
			loop {
				for _ in 0..50 {
					pin.set_high();
					thread::sleep(n);
					pin.set_low();
					thread::sleep(p - n);
				}
				for _ in 0..50 {
					pin.set_high();
					thread::sleep(l);
					pin.set_low();
					thread::sleep(p - l);
				}
				for _ in 0..50 {
					pin.set_high();
					thread::sleep(h);
					pin.set_low();
					thread::sleep(p - h);
				}
			}
		});
	}

	loop { }
}
*/
