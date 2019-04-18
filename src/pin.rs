//! Utilities for working with GPIO pins.
use std::time::Duration;
use std::{fmt, io::Error as IoError};

#[cfg(all(feature = "stub", feature = "use_rppal"))]
compile_error!("Cannot stub and use rppal simultaneously");

/// Trait representing an output device capable of (software) PWM.
pub trait Pwm {
    /// Sets the pulse width and period for the device.
    fn set_pwm(&mut self, period: Duration, pulse_width: Duration) -> Result<(), Error>;
}

/// Trait representing a general output device.
pub trait Out {
    /// Sets the output device high.
    fn set_high(&mut self);
    /// Sets the output device low.
    fn set_low(&mut self);
    /// Sets the output device high/low.
    fn set(&mut self, value: bool) {
        if value {
            self.set_high()
        } else {
            self.set_low()
        }
    }
}

#[cfg(not(feature = "stub"))]
mod gpio {
    use super::{Error, Out, Pwm};
    use lazy_static::lazy_static;
    pub(crate) use rppal::gpio::{Gpio, OutputPin};
    use std::time::Duration;
    lazy_static! {
        pub static ref GPIO: Gpio = Gpio::new().unwrap();
    }
    pub(crate) fn pin(number: u8) -> Result<OutputPin, Error> {
        Ok(GPIO.get(number).map(|pin| pin.into_output())?)
    }
    impl Pwm for OutputPin {
        fn set_pwm(&mut self, period: Duration, pulse_width: Duration) -> Result<(), Error> {
            if pulse_width == Duration::new(0, 0) {
                self.clear_pwm()?;
            } else {
                log::trace!("Setting output pin pulse width to {:?}", pulse_width);
                self.set_pwm(period, pulse_width)?;
            }
            Ok(())
        }
    }
    impl Out for OutputPin {
        fn set_high(&mut self) {
            Self::set_high(self);
        }
        fn set_low(&mut self) {
            Self::set_low(self);
        }
    }
}

#[cfg(feature = "stub")]
mod stub {
    use super::{Error, Out, Pwm};
    use std::time::Duration;
    #[derive(Debug)]
    pub(crate) struct Stub;
    impl Pwm for Stub {
        fn set_pwm(&mut self, _: Duration, _: Duration) -> Result<(), Error> {
            Ok(())
        }
    }
    impl Out for Stub {
        fn set_high(&mut self) {}
        fn set_low(&mut self) {}
    }
}

/// GPIO operation error type.
#[derive(Debug)]
pub enum Error {
    /// The model of the device couldn't be identified.
    Model,
    /// The given pin is in use or unavailable.
    Unavailable(u8),
    /// Permission was denied to access the device.
    Permission(String),
    /// An I/O error occured.
    Io(IoError),
    /// A thread panicked.
    Panic,
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Self {
        Error::Io(err)
    }
}

#[cfg(feature = "use_rppal")]
use rppal::gpio::Error as RppalError;
#[cfg(feature = "use_rppal")]
impl From<RppalError> for Error {
    fn from(err: RppalError) -> Self {
        match err {
            RppalError::Io(err) => Error::Io(err),
            RppalError::UnknownModel => Error::Model,
            RppalError::PinNotAvailable(pin) => Error::Unavailable(pin),
            RppalError::PermissionDenied(path) => Error::Permission(path),
            RppalError::ThreadPanic => Error::Panic,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(err) => err.fmt(f),
            Error::Model => write!(f, "Unknown Pi model/SoC"),
            Error::Unavailable(pin) => write!(f, "Pin {} unavailable (in use or nonexistent)", pin),
            Error::Permission(path) => write!(f, "Permission denied when accessing path {}", path),
            Error::Panic => write!(f, "Thread panicked."),
        }
    }
}

impl std::error::Error for Error {}

/// Represents a GPIO pin.
#[derive(Debug)]
pub struct Pin {
    pub(crate) number: u16,
    #[cfg(not(feature = "stub"))]
    output: self::gpio::OutputPin,
    #[cfg(feature = "stub")]
    output: self::stub::Stub,
}

impl Pin {
    /// Attempts to create an output Pin struct on the given pin number.
    #[cfg(not(feature = "stub"))]
    pub fn try_new(number: u16) -> Result<Self, Error> {
        Ok(Self {
            output: gpio::pin(number as u8)?,
            number,
        })
    }
    /// Creates a stub Pin output struct on the given pin number.
    #[cfg(feature = "stub")]
    pub fn try_new(number: u16) -> Result<Self, Error> {
        log::info!("Using a stub for GPIO; writes will be ignored");
        Ok(Self {
            output: self::stub::Stub,
            number,
        })
    }
    /// Sets the pin to the desired state.
    pub fn set(&mut self, high: bool) {
        self.output.set(high);
    }
    /// Sets the pin high.
    pub fn set_high(&mut self) {
        self.set(true)
    }
    /// Sets the pin low.
    pub fn set_low(&mut self) {
        self.set(false)
    }
}

impl Out for Pin {
    fn set_high(&mut self) {
        self.output.set_high()
    }
    fn set_low(&mut self) {
        self.output.set_low()
    }
}

impl Pwm for Pin {
    fn set_pwm(&mut self, period: Duration, pulse_width: Duration) -> Result<(), Error> {
        self.output.set_pwm(period, pulse_width)?;
        Ok(())
    }
}
