use std::time::Duration;
use std::{fmt, thread};
use std::io::Error as IoError;
use gpio::sysfs::SysFsGpioOutput;
use gpio::GpioOut;

#[allow(dead_code)]
/// Enables stubbing for testing purposes.
/// Since testing can occur outside of the testing environment (a la documentation tests), this uses `feature = "stub"` instead of the `test` config flag.
pub struct GpioOutputStub {
	pub pin: u16,
	pub is_high: bool
}

#[allow(dead_code)]
impl GpioOutputStub {
	fn new(pin: u16) -> Self {
		Self { pin, is_high: false }
	}
	fn set_high(&mut self) -> Result<(), IoError> {
		self.is_high = true;
		Ok(())
	}
	fn set_low(&mut self) -> Result<(), IoError> {
		self.is_high = false;
		Ok(())
	}
}

pub type PinResult = Result<(), IoError>;

/// Represents an object managing a GPIO pin (a very thin wrapper).
/// # Notes
/// This object is stub-able (for use in testing environments without GPIO access).
/// To enable stubbing, use the config flag `stub`.
pub struct Pin {
	#[cfg(feature = "stub")]
	output: GpioOutputStub,
	/// The backing object through which communication with the GPIO actually happens.
	#[cfg(not(feature = "stub"))]
	output: SysFsGpioOutput,
	/// The pin number with which this instance is associated.
	pub number: u16,
	/// Encodes the underlying pin's state (`true` for high, `false` for low).
	pub is_high: bool
}

impl Pin {

	#[cfg(feature = "stub")]
	pub fn new(number: u16) -> Self {
		Self {
			output: GpioOutputStub::new(number),
			number,
			is_high: false
		}
	}

	/// Constructs a new `Pin` managing the GPIO pin `number`.
	#[cfg(not(feature = "stub"))]
	pub fn new(number: u16) -> Self {
		Self {
			output: SysFsGpioOutput::new(number).unwrap(),
			number,
			is_high: false
		}
	}

	/// Sets the pin high.
	pub fn set_high(&mut self) -> PinResult {
		self.is_high = true;
		self.output.set_high()
	}

	/// Sets the pin low.
	pub fn set_low(&mut self) -> PinResult {
		self.is_high = false;
		self.output.set_low()
	}

	/// Performs a single cycle of a wave (for use with PWM) using busy loops.
	/// # Panics
	/// This method will panic if `width` >= `total` (that is, the wave is as wide as the period or wider), as this badly malformed of input is likely not somehing that can be recovered from.
	///
	/// In general, this method panics if the input is malformed, returning `Result::Err` only in the case of errors from the GPIO underpinnings.
	pub fn do_wave(&mut self, width: Duration, total: Duration) -> PinResult {
		assert!(total > width, "Wave pulse of {:?} exceeds period of {:?}.", width, total);
		self.set_high()?;
		thread::sleep(width);
		self.set_low()?;
		thread::sleep(total - width);
		Ok(())
	}

}

impl fmt::Debug for Pin {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Pin {} ({})", self.number, self.is_high)
	}
}
