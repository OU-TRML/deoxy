//! Handles user-given configuration: motor types, pins, etc.

use std::error::Error as StdError;
use std::fmt;
use std::fs::File;
use std::io::{Error as IoError, Read};
use std::path::Path;
use std::str::FromStr;

use toml;

/// Represents a deserialization error.
#[derive(Debug)]
pub enum Error {
    /// An error occured while parsing a TOML string.
    TomlError(toml::de::Error),
    /// An I/O error occured.
    IoError(IoError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {}",
            match self {
                Error::TomlError(_) => "TOML parse error",
                Error::IoError(_) => "I/O error",
            },
            self.description()
        )
    }
}

impl From<toml::de::Error> for Error {
    fn from(error: toml::de::Error) -> Self {
        Error::TomlError(error)
    }
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Error::IoError(error)
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self {
            Error::TomlError(err) => err.description(),
            Error::IoError(err) => err.description(),
        }
    }
}

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
    ///
    /// # Examples
    /// ```
    /// # extern crate deoxy;
    /// # use std::str::FromStr;
    /// # use deoxy::config::{Config, MotorSpec};
    /// let cfg = Config::from_str("[[motors]]\npin = 17\nrange = [1, 2]\nperiod = 20").unwrap();
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
    /// let cfg = Config::from_str("[[motors]]\npin = 17\nrange = [1, 2]\nperiod = 20").unwrap();
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
    /// let cfg = Config::from_str("[[motors]]\npin = 17\nrange = [1, 2]\nperiod = 20").unwrap();
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
    /// let cfg = Config::from_str("[[motors]]\npin = 17\nrange = [1, 2]\nperiod = 20").unwrap();
    /// let motors = cfg.motors();
    /// let motor = &motors[0];
    /// assert_eq!(motor.get_period(), 20);
    /// ```
    pub fn get_period(&self) -> u64 {
        self.period
    }
}

impl<'a> Config {
    /// Fetches configuration from the specified location.
    pub fn from_path<P: AsRef<Path>>(path: &P) -> Result<Self, Error> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        let _bytes = file.read_to_string(&mut contents)?;
        let cfg = Self::from_str(&contents)?;
        Ok(cfg)
    }

    /// All motors specified by the configuration.
    pub fn motors(&'a self) -> &'a [MotorSpec] {
        &self.motors
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
