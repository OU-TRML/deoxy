//! A buffer-exchange crate.

#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]
#![deny(missing_docs)]

use std::thread;
use std::time::Duration;

#[macro_use]
extern crate failure;
extern crate gpio;
#[macro_use]
extern crate serde_derive;
extern crate toml;

mod angle;
pub mod communication;
pub mod config;
pub mod io;
pub mod motion;

use communication::{Action, Coordinator};
use config::Config;
#[allow(unused_imports)]
use io::{GpioOutputStub, Pin};

fn send_open(coord: &Coordinator, index: usize) -> Result<(), std::sync::mpsc::SendError<Action>> {
    coord.channels[index].send(Action::Open(Some(Duration::from_secs(2))))
}

/// Exactly what it says on the tin (for now).
#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
pub fn main(config: Config) {
    let mgr = Coordinator::from(&config);
    for i in 0..10 {
        send_open(&mgr, i).unwrap();
    }
    thread::sleep(Duration::from_millis(3_000));
}
