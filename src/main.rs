extern crate deoxy;
use deoxy::communication::{Hub, Message};
use std::env;
mod flags;
use flags::Flag;
mod config;
use config::Config;

macro_rules! print_help {
	($fmt:expr, $($arg:tt)*) => ({
		println!($fmt, $($arg),*); // What the heck is this magic
		print_help!();
	});
	($arg:tt) => ({
		print_help!("{}", $arg);
	});
	() => ({
		println!("usage: {name} [help] [--config-path <path>]", name=env::args().nth(0).unwrap_or("deoxy".to_string()));
	});
}

fn main() {
	let args = env::args();
	match Flag::from(args) {
		Ok(flags) => {
			let config = flags.iter().map(|x| match x { &Flag::ConfigPath(ref path) => Some(path), _ => None }).last().map(|ref path| Config::read_or_default(path.unwrap().as_str())).unwrap_or_default();
			let order = config.order;
			let hub = Hub::with_threads(order as u8);
			let mut queue = Vec::with_capacity(order);
			for i in 0..order {
				queue.push(hub.send(i, Message::Debug(format!("Hello, thread {}!", i))));
			}
			loop { } // Everything else happens on other threads, but we have to keep the main thread alive.
		}, Err(text) => { // Failed to parse the passed arguments; gently correct the user by showing the help message.
			if let Some(message) = text {
				print_help!(message);
			} else {
				print_help!();
			}
		}
	}
}
