use std::sync::mpsc;
use std::time::Duration;
use std::{env, fmt, thread};
use std::io::Error as IoError;

#[allow(unused_imports)]
extern crate gpio;
use self::gpio::sysfs::SysFsGpioOutput;
use self::gpio::GpioOut;

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

/// Represents a motor mounted on the board.
pub struct Motor {
	/// The underlying `Pin` instance which manages the state of the GPIO pin to which the motor is attached.
	pin: Pin,
	/// The current (cached) pulse width for the motor signal.
	pulse_width: Duration,
	/// The motor's constant, characterisic period.
	pub period: Duration,
	/// The range of pulse widths this motor supports (used to calculate appropriate widths for neutral and anti-neutral positions).
	signal_range: (Duration, Duration)
}
// TODO: From methods
impl Motor {
	/// Constructs a new `Motor` with the given properties.
	/// # Notes
	/// You are discouraged from using this method outside of testing environments; you should instead use appropriate `From` methods (see below).
	/// This method will also instantiate an underlying `Pin`.
	/// The resulting object will have a pulse width of 0 until one is specified or the [`_loop`](#method._loop) method automatically generates one (if applicable).
	pub fn new(pin_number: u16, period: Duration, signal_range: (Duration, Duration)) -> Self {
		Self {
			pin: Pin::new(pin_number),
			period,
			pulse_width: Duration::new(0, 0),
			signal_range
		}
	}
	/// The main method which manages the motor.
	/// Once this method is invoked, the motor will constantly receive a signal with the characteristic period until an I/O error occurs. The duty cycle can be varied using other methods.
	/// # Errors
	/// If this method returns, it will **always** be a `Result::Err<std::io::Error>` describing what went wrong.
	/// # Notes
	/// If this method is invoked and the currently set pulse width is 0 (as would likely happen immediately after instantiation), the pulse width is set to the (calculated) neutral position instead. To disable this behavior, use the config flag `no_neutral_correction`.
	pub fn _loop(&mut self) -> PinResult {
		if !cfg!(no_neutral_correction) && self.pulse_width == Duration::new(0, 0) {
			self.pulse_width = (self.signal_range.0 + self.signal_range.1) / 2;
		}
		loop {
			self.pin.do_wave(self.pulse_width, self.period)?;
		}
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

fn main() {
	let args = env::args().collect::<Vec<_>>();
	let u = args.get(1).map(|v| v.parse::<u32>().unwrap_or(1_700)).unwrap_or(1_700);
	let (tx, rx) = mpsc::channel();
	let child = thread::spawn(move || {
		let pin = rx.recv().unwrap();
		if cfg!(feature = "stub") {
			println!("Emulating running against pin {} with width {} µs", pin, u);
		} else {
			println!("Running against pin {} with width {} µs", pin, u);
		}
		let mut pin = Pin::new(pin);
		let (width, total) = (Duration::new(0, u * 1000), Duration::from_millis(20));
		for _ in 0..1_000 {
			let _ = pin.do_wave(width, total).unwrap();
		}
	});
	let _ = tx.send(17);
	let result = child.join();
	if let Err(err) = result {
		println!("Error: {:?}", err);
	}
}
