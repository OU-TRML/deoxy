extern crate toml;
use self::toml::Value;
use self::toml::value::Table;
use std::default::Default;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::time::Duration;
extern crate regex;
use self::regex::Regex;

#[derive(Debug)]
pub enum MotorType {
	HS_645MG,
	Custom(Duration, (Duration, Duration)) // period, range
}

impl MotorType {

	fn period(&self) -> Duration {
		match *self {
			MotorType::Custom(period, _) => period,
			MotorType::HS_645MG => Duration::from_millis(20)
		}
	}

	fn range(&self) -> (Duration, Duration) {
		match *self {
			MotorType::Custom(_, range) => range,
			MotorType::HS_645MG => (Duration::new(0, 900_000), Duration::new(0, 2_100_000))
		}
	}

	fn try_from(value: &Table) -> Option<Self> {
		if let Some(t) = value.get("type").and_then(|t| t.as_str()) {
			match &*t.to_lowercase() {
				p if Regex::new(r"hs(-|_)?645mg").unwrap().is_match(p) => Some(MotorType::HS_645MG),
				"custom" => {
					None // TODO: Support custom types
				}, _ => None
			}
		} else {
			None
		}
	}

}

#[derive(Debug)]
pub struct MotorSpec {
	port: u8,
	variant: MotorType
}

impl MotorSpec {
	fn range(&self) -> (Duration, Duration) { self.variant.range() }
	fn period(&self) -> Duration { self.variant.period() }
	fn try_from(value: &Table) -> Option<Self> {
		if let Some(port) = value.get("port").and_then(|p| p.as_integer()) {
			MotorType::try_from(value).map(|t| Self { port: port as u8, variant: t }) // Nice
		} else {
			None
		}
	}
}
/// An opaque type used solely to load the motor layout from a configuration file.
#[derive(Debug)]
pub struct Config {
	/// The number of motors configured.
	pub order: usize,
	pub motors: Vec<MotorSpec>
}

impl Config {
	/// Attempts to parse the configuration file at `path` and returns either the resulting read configuration or the default configuration.
	pub fn read_or_default(path: &str) -> Self {
		let mut text = String::new();
		let spec = File::open(path).map(|mut file| file.read_to_string(&mut text)).map(|_| text.parse::<Value>());
		match spec {
			Ok(Ok(value)) => Self::from_tree(value),
			_ => Self::default()
		}
	}

	pub fn from_tree(value: Value) -> Self {
		if let Some(nodes) = value.get("motors").and_then(|n| n.as_array()) {
			let order = nodes.len();
			let mut motors = Vec::with_capacity(order);
			for node in nodes {
				if let Some(motor) = node.as_table().and_then(|table| MotorSpec::try_from(table)) {
					motors.push(motor);
				} else {
					// Ignore failed conversion (TODO: At least alert the user)
				}
			}
			Self {
				order,
				motors
			}
		} else {
			Self::default()
		}
	}

}

impl Default for Config {
	fn default() -> Self {
		let spec = include_str!("../config-example.toml").parse::<Value>().expect("Failed to parse default configuration; compile-time error?");
		Self::from_tree(spec)
	}
}