//! Utilities for working with GPIO pins.
use std::{fmt, io::Error as IoError};

use gpio::{dummy::DummyGpioOut, sysfs::SysFsGpioOutput, GpioOut};

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
/// An actual pin type (non-stubbed).
pub type Pin = Out<SysFsGpioOutput>;
#[allow(dead_code)]
pub type Stub<F> = Out<DummyGpioOut<F>>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Out<T>
where
    T: GpioOut,
{
    pub(crate) number: u16,
    output: T,
}

impl Pin {
    /// Attempts to create an output Pin struct on the given pin number.
    pub fn try_new(number: u16) -> std::result::Result<Self, Error> {
        Ok(Self {
            output: SysFsGpioOutput::open(number)?,
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
