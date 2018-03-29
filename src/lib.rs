use std::time::Duration;

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
use communication::Coordinator;
use config::Config;

pub fn main(config: Config) {
	let mgr = Coordinator::from(config.get_motors());
}
