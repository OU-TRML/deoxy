use std::path::Path;
use std::fs::File;
use std::io::Read;

use toml;

#[derive(Serialize, Deserialize)]
pub struct Config {
	motors: Vec<MotorSpec>
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MotorSpec {
	pin: u16,
	range: [u32; 2], // µs
	period: u64 // ms
}

impl MotorSpec {

	pub fn get_pin(&self) -> u16 {
		self.pin
	}

	pub fn get_min(&self) -> u32 {
		self.range[0]
	}

	pub fn get_max(&self) -> u32 {
		self.range[0]
	}

	pub fn get_period(&self) -> u64 {
		self.period
	}

}

impl Config {

	pub fn from_str(str: &str) -> Result<Self, ()> {
		toml::from_str(str).or(Err(()))
	}

	pub fn from_path(path: &Path) -> Result<Self, ()> {
		if let Ok(mut file) = File::open(path) {
			let mut contents = String::new();
			match file.read_to_string(&mut contents) {
				Ok(_) => Self::from_str(&contents),
				_ => Err(())
			}
		} else {
			Err(())
		}
	}

	pub fn from_path_string(str: &str) -> Result<Self, ()> {
		Self::from_path(Path::new(str))
	}

	pub fn get_motors(&self) -> Vec<MotorSpec> {
		self.motors.clone() // TODO: Avoid copy
	}

}

impl Default for Config {
	fn default() -> Self {
		Self::from_str(include_str!("../config-example.toml")).unwrap() // TODO: Possibly handle error?
	}
}