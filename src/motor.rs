extern crate gpio;

use std::time::{Duration, Instant};

use self::gpio::{GpioOut, sysfs};
use std::io::Error;
use std::thread;
use std::sync::{Mutex, Arc};

// https://servodatabase.com/servo/hitec/hs-645mg

/// Represents the range of possible values a motor's pulse width can take (as `(min, max)`).
pub type MotorRange = (Duration, Duration);

/// Possible errors encountered when changing the motor's pulse width.
pub enum MotorError {
	/// The user tried to change the pulse width to a duration lying outside the set bounds.
	OutOfBounds,
	/// An error occured in sending the required message. If applicable, more information is supplied in the associated `String`.
	CommunicationError(Option<String>)
}

struct Pin {
	pin: u8,
	high: bool,
	#[cfg(not(test))]
	output: sysfs::SysFsGpioOutput
}

impl Pin {

	fn new(pin: u8) -> Self {
		Self {
			pin,
			high: false,
			#[cfg(not(test))]
			output: sysfs::SysFsGpioOutput::new(pin as u16).unwrap()
		}
	}

	#[cfg(not(test))]
	#[inline(always)]
	pub fn set_high(&mut self) -> Result<(), Error> {
		self.high = true;
		self.output.set_high()
	}

	#[cfg(test)]
	#[inline(always)]
	pub fn set_high(&mut self) -> Result<(), Error> {
		self.high = true;
		Ok(())
	}

	#[cfg(not(test))]
	#[inline(always)]
	pub fn set_low(&mut self) -> Result<(), Error> {
		self.high = false;
		self.output.set_low()
	}

	#[cfg(test)]
	#[inline(always)]
	pub fn set_low(&mut self) -> Result<(), Error> {
		self.high = false;
		Ok(())
	}

	pub fn set(&mut self, high: bool) -> Result<(), Error> {
		self.high = high;
		if high {
			self.set_high()
		} else {
			self.set_low()
		}
	}
}

type ScheduledChange = (Instant, bool);
type ScheduledChanges = Vec<ScheduledChange>;

/// Represents a hardware motor.
///
/// Motors are given all the necessary configuration information to manage their own position and communication and provide a high-level interface to accomplish related tasks.
pub struct Motor {
	pin: Arc<Mutex<Pin>>,
	/// The current pulse width.
	pulse_width: Duration,
	period: Duration, // 20 ms
	range: MotorRange,
	queue: Arc<Mutex<ScheduledChanges>>
}

impl Motor {

	/// Constructs a new motor on the given pin which has the given period.
	///
	/// `range` takes the format `(minimum, maximum)`.
	pub fn new(pin: u8, period: Duration, range: MotorRange) -> Self {
		let mut motor = Self {
			pin: Arc::new(Mutex::new(Pin::new(pin))),
			period,
			range,
			pulse_width: Duration::new(0, 0),
			queue: Arc::new(Mutex::new(vec![(Instant::now(), true)])), // Set high immediately (TODO: Remove)
		};
		// let _ = motor.set_neutral();
		motor
	}

	/// The minimum usable pulse width, as specified upon creation.
	/// # Examples
	/// ```rust,no_run
	/// use std::time::Duration;
	/// use deoxy::Motor;
	/// let motor = Motor::new(0, Duration::from_millis(20), (Duration::new(0, 900_000), Duration::new(0, 1_200_000)));
	/// assert_eq!(motor.min(), Duration::new(0, 900_000));
	/// ```
	pub fn min(&self) -> Duration {
		self.range.0
	}

	/// The maximum usable pulse width, as specified upon creation.
	/// # Examples
	/// ```rust,no_run
	/// use std::time::Duration;
	/// use deoxy::Motor;
	/// let motor = Motor::new(0, Duration::from_millis(20), (Duration::new(0, 900_000), Duration::new(0, 1_200_000)));
	/// assert_eq!(motor.max(), Duration::new(0, 1_200_000));
	/// ```
	pub fn max(&self) -> Duration {
		self.range.1
	}

	/// Sets the pulse width to the center of the possible range, bringing it to the neutral position.
	pub fn set_neutral(&mut self) -> Result<(), MotorError> {
		let (min, max) = (self.min(), self.max());
		self.set_pulse_width((min + max) / 2)
	}

	/// Attempts to set the pulse width to the given duration.
	pub fn set_pulse_width(&mut self, pulse_width: Duration) -> Result<(), MotorError> {
		if pulse_width < self.min() || pulse_width > self.max() {
			Err(MotorError::OutOfBounds)
		} else {
			self.pulse_width = pulse_width;
			Ok(())
		}
	}

	fn add_pulses(&mut self, number: u32) {
		let mut queue = self.queue.lock().unwrap();
		let last = queue.last().map(|t| t.0).unwrap_or(Instant::now());
		let offset = self.period;
		let width = self.pulse_width;
		for i in 0..number {
			let target = last + offset * i;
			queue.push((target, true));
			queue.push((target + width, false));
		}
	}

	pub fn _loop(&self) {
		loop { // TODO: Allow exiting loop
			let queue = self.queue.clone(); // TODO: Is this necessary?
			let pin = self.pin.clone();
			let result = thread::spawn(move || {
				loop {
					if let Some(action) = queue.lock().unwrap().pop() {
						let now = Instant::now();
						let value = action.1;
						while now < action.0 { // TODO: Perhaps loop with a break?
							// No-op (busy loop)
						}
						if let Err(err) = pin.lock().unwrap().set(value) {
							panic!(err);
						}
					}
				}
			}).join();
			if let Err(message) = result {
				println!("Child thread crashed ({:?}); respawning.", message); // TODO: stderr
			}
		}
	}

}