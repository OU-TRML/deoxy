//! Handles user-given configuration: motor types, pins, etc.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use toml;

/// Holds the configuration for the given instance.
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    motors: Vec<MotorSpec>,
}

/// Fully specifies a motor.
// Again, to prevent multiple things on one pin, we fail to implement Copy.
#[allow(missing_copy_implementations)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MotorSpec {
    pin: u16,
    range: [u32; 2], // µs
    period: u64,     // ms
}

impl MotorSpec {
    /// Returns the pin the motor is attached to.
    pub fn get_pin(&self) -> u16 {
        self.pin
    }

    /// Returns the minimum useful duty cycle.
    pub fn get_min(&self) -> u32 {
        self.range[0]
    }

    /// Returns the maximum useful duty cycle.
    pub fn get_max(&self) -> u32 {
        self.range[0]
    }

    /// Returns the period of the motor.
    pub fn get_period(&self) -> u64 {
        self.period
    }
}

impl<'a> Config {
    /// Parses the passed string into a configuration.
    pub fn from_str(str: &str) -> Result<Self, ()> {
        toml::from_str(str).or(Err(()))
    }

    /// Fetches configuration from the specified location.
    pub fn from_path<P: AsRef<Path>>(path: &P) -> Result<Self, ()> {
        if let Ok(mut file) = File::open(path) {
            let mut contents = String::new();
            match file.read_to_string(&mut contents) {
                Ok(_) => Self::from_str(&contents),
                _ => Err(()),
            }
        } else {
            Err(())
        }
    }

    /// All motors specified by the configuration.
    pub fn motors(&'a self) -> &'a [MotorSpec] {
        &self.motors
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::from_str(include_str!("../config-example.toml")).unwrap() // TODO: Possibly handle error?
    }
}
