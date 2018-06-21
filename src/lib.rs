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

pub fn main(config: Config) {
	let mgr = Coordinator::from(config.get_motors());
}
