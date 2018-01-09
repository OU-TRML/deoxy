extern crate deoxy;
use deoxy::Hub;
use deoxy::Message;

fn main() {
	let hub = Hub::new(3);
	let mut queue = vec![];
	for i in 0..3 {
		queue.push(hub.send(i, Message::Debug(format!("Hello, thread {}!", i))));
	}
	loop { }
}
