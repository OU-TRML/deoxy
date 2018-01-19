extern crate deoxy;
use deoxy::communication::{Hub, Message};
use std::time::Duration;
use deoxy::config::{Config, MotorSpec, MotorType};

fn main() {
	let hub = Hub::new(Config { order: 1, motors: vec![MotorSpec { pin: 11, variant: MotorType::HS_645MG }]});
	hub.send(0, Message::Debug("Ping".to_string()));
	let _result = hub.send(0, Message::SetPulseWidthForDuration(Duration::from_millis(1), Duration::from_millis(20_000)));
	hub.send(0, Message::Debug("Done.".to_string()));
	loop { }
}