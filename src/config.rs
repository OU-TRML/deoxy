extern crate toml;
use self::toml::Value;
use std::default::Default;
use std::io;
use std::io::prelude::*;
use std::fs::File;

/// An opaque type used solely to load the motor layout from a configuration file.
pub struct Config {
	tree: Value
}

impl Config {
	/// Attempts to parse the configuration file at `path` and returns either the resulting read configuration or the default configuration.
	pub fn read_or_default(path: &str) -> Self {
		let mut text = String::new();
		let spec = File::open(path).map(|mut file| file.read_to_string(&mut text)).map(|_| text.parse::<Value>());
		match spec {
			Ok(Ok(value)) => {
				Self {
					tree: value
				}
			}, _ => {
				Self::default()
			}
		}
	}

	/// Returns the number of motors configured.
	pub fn order(&self) -> usize {
		if let Some(nodes) = self.tree.get("motors").and_then(|n| n.as_array()) {
			nodes.len()
		} else {
			0
		}
	}

}

impl Default for Config {
	fn default() -> Self {
		let spec = include_str!("../config-example.toml").parse::<Value>().expect("Failed to parse default configuration; compile-time error?");
		Self {
			tree: spec
		}
	}
}