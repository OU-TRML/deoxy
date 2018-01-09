use std::time::Duration;

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

/// A `Motor` represents a hardware motor.
///
/// Motors are given all the necessary configuration information to manage their own position and communication and provide a high-level interface to accomplish related tasks.
pub struct Motor {
	pin: u8,
	/// The current pulse width.
	pulse_width: Duration,
	period: Duration, // 20 ms
	range: MotorRange
}

impl Motor {

	/// Constructs a new motor on the given pin which has the given period.
	///
	/// `range` takes the format `(minimum, maximum)`.
	pub fn new(pin: u8, period: Duration, range: MotorRange) -> Self {
		let mut motor = Self {
			pin,
			period,
			range,
			pulse_width: Duration::new(0, 0)
		};
		let _ = motor.set_neutral();
		motor
	}

	pub fn min(&self) -> Duration {
		self.range.0
	}

	pub fn max(&self) -> Duration {
		self.range.1
	}

	pub fn set_neutral(&mut self) -> Result<(), MotorError> {
		let (min, max) = (self.min(), self.max());
		self.set_pulse_width((min + max) / 2)
	}

	pub fn set_pulse_width(&mut self, pulse_width: Duration) -> Result<(), MotorError> {
		if pulse_width < self.min() || pulse_width > self.max() {
			Err(MotorError::OutOfBounds)
		} else {
			self.pulse_width = pulse_width;
			Ok(())
		}
	}

}