//! Handles user-given configuration: motor types, pins, etc.

use std::fs::File;
use std::io::{Error as IoError, Read};
use std::path::Path;
use std::str::FromStr;

use failure::Error;
use toml;

/// Represents a configuration deserialization error.
#[derive(Debug, Fail)]
pub enum ConfigError {
    /// An error occured while parsing a TOML string.
    #[fail(display = "TOML error: {}", error)]
    TomlError {
        /// The underlying TOML deserialization error (cause).
        error: toml::de::Error,
    },
    /// An I/O error occured.
    #[fail(display = "I/O error: {}", error)]
    IoError {
        /// The underlying I/O error (cause).
        error: IoError,
    },
}

/// Holds the configuration for the given instance.
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    motors: Vec<MotorSpec>,
    pump: PumpSpec,
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
    ///
    /// # Examples
    /// ```
    /// # extern crate deoxy;
    /// # use std::str::FromStr;
    /// # use deoxy::config::{Config, MotorSpec};
    /// let cfg = Config::from_str("[pump]\npins=[1,2,3,4]\n[[motors]]\npin = 17\nrange = [1, 2]\nperiod = 20").unwrap();
    /// let motors = cfg.motors();
    /// let motor = &motors[0];
    /// assert_eq!(motor.get_pin(), 17);
    /// ```
    pub fn get_pin(&self) -> u16 {
        self.pin
    }

    /// Returns the minimum useful duty cycle.
    ///
    /// # Examples
    /// ```
    /// # extern crate deoxy;
    /// # use std::str::FromStr;
    /// # use deoxy::config::{Config, MotorSpec};
    /// let cfg = Config::from_str("[pump]\npins=[1,2,3,4]\n[[motors]]\npin = 17\nrange = [1, 2]\nperiod = 20").unwrap();
    /// let motors = cfg.motors();
    /// let motor = &motors[0];
    /// assert_eq!(motor.get_min(), 1);
    /// ```
    pub fn get_min(&self) -> u32 {
        self.range[0]
    }

    /// Returns the maximum useful duty cycle.
    ///
    /// # Examples
    /// ```
    /// # extern crate deoxy;
    /// # use std::str::FromStr;
    /// # use deoxy::config::{Config, MotorSpec};
    /// let cfg = Config::from_str("[pump]\npins=[1,2,3,4]\n[[motors]]\npin = 17\nrange = [1, 2]\nperiod = 20").unwrap();
    /// let motors = cfg.motors();
    /// let motor = &motors[0];
    /// assert_eq!(motor.get_max(), 2);
    /// ```
    pub fn get_max(&self) -> u32 {
        self.range[1]
    }

    /// Returns the period of the motor.
    ///
    /// # Examples
    /// ```
    /// # extern crate deoxy;
    /// # use std::str::FromStr;
    /// # use deoxy::config::{Config, MotorSpec};
    /// let cfg = Config::from_str("[pump]\npins=[1,2,3,4]\n[[motors]]\npin = 17\nrange = [1, 2]\nperiod = 20").unwrap();
    /// let motors = cfg.motors();
    /// let motor = &motors[0];
    /// assert_eq!(motor.get_period(), 20);
    /// ```
    pub fn get_period(&self) -> u64 {
        self.period
    }
}

/// Fully specifies a pump.
// To prevent multiple things on one pin, we fail to implement Copy.
#[allow(missing_copy_implementations)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PumpSpec {
    /// The pins the H-bridge uses. See the documentation for `Pump` for more details.
    pub pins: [u16; 4],
    // TODO (#15): type: PumpControlType (H-bridge, DPDT, whatever else)
}

impl Config {
    /// Fetches configuration from the specified location.
    pub fn from_path<P: AsRef<Path>>(path: &P) -> Result<Self, Error> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        let _bytes = file.read_to_string(&mut contents)?;
        let cfg = Self::from_str(&contents)?;
        Ok(cfg)
    }

    /// All motors specified by the configuration.
    pub fn motors(&self) -> &[MotorSpec] {
        &self.motors
    }

    /// The specification for the pump given by this configuration.
    pub fn pump(&self) -> &PumpSpec {
        &self.pump
    }
}

impl FromStr for Config {
    type Err = toml::de::Error;
    /// Parses the passed TOML string into a configuration.
    fn from_str(str: &str) -> Result<Self, Self::Err> {
        toml::from_str(str)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::from_str(include_str!("../config-example.toml")).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use Config;
    #[test]
    fn test_default_config() {
        let _cfg = Config::default();
    }
}
