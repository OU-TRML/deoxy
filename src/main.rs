extern crate deoxy;
use deoxy::communication::{Hub, Message};
use deoxy::config::Config;
use std::env;

fn main() {
	// Config file support
	let mut args = env::args();
	let config = if let Some(index) = args.position(|arg| arg == "--config-path" || arg == "-c" || arg == "--c") {
		if let Some(path) = args.nth(index + 1) {
			Config::read_or_default(&path)
		} else {
			Config::default()
		}
	} else {
		Config::default()
	};
	let order = config.order();
	let hub = Hub::new(order as u8);
	let mut queue = Vec::with_capacity(order);
	for i in 0..order {
		queue.push(hub.send(i, Message::Debug(format!("Hello, thread {}!", i))));
	}
	loop { }
}
