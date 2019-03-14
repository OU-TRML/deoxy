//! Utilities for working with GPIO pins.
use std::{fmt, io::Error as IoError};

#[cfg_attr(feature = "stub", allow(unused_imports))]
use gpio::{sysfs::SysFsGpioOutput, GpioOut};

/// GPIO operation error type.
#[derive(Debug)]
pub enum Error {
    /// An I/O error occured.
    Io(IoError),
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Self {
        Error::Io(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Error::Io(err) = self;
        err.fmt(f)
    }
}

impl std::error::Error for Error {}

/// Pin result type, returning pin state or a write error.
pub type Result = std::result::Result<bool, Error>;

/// Represents a GPIO pin.
#[derive(Debug)]
pub struct Pin {
    pub(crate) number: u16,
    #[cfg(not(feature = "stub"))]
    output: SysFsGpioOutput,
    #[cfg(feature = "stub")]
    output: self::stub::StubOutput,
}

#[cfg(feature = "stub")]
mod stub {
    use std::io::Error as IoError;
    #[derive(Debug)]
    pub struct StubOutput {}

    impl gpio::GpioOut for StubOutput {
        type Error = IoError;
        fn set_high(&mut self) -> Result<(), IoError> {
            Ok(())
        }
        fn set_low(&mut self) -> Result<(), IoError> {
            Ok(())
        }
    }
}

impl Pin {
    /// Attempts to create an output Pin struct on the given pin number.
    #[cfg(not(feature = "stub"))]
    pub fn try_new(number: u16) -> std::result::Result<Self, Error> {
        Ok(Self {
            output: SysFsGpioOutput::open(number)?,
            number,
        })
    }
    /// Creates a stub Pin output struct on the given pin number.
    #[cfg(feature = "stub")]
    pub fn try_new(number: u16) -> std::result::Result<Self, Error> {
        Ok(Self {
            output: self::stub::StubOutput {},
            number,
        })
    }
    /// Sets the pin to the desired state.
    pub fn set(&mut self, high: bool) -> Result {
        self.output.set_value(high)?;
        Ok(high)
    }
    /// Sets the pin high.
    pub fn set_high(&mut self) -> Result {
        self.set(true)
    }
    /// Sets the pin low.
    pub fn set_low(&mut self) -> Result {
        self.set(false)
    }
}
