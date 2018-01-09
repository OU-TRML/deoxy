use std::time::Duration;

// https://servodatabase.com/servo/hitec/hs-645mg

pub enum MotorError {
	OutOfBounds,
	CommunicationError(Option<String>)
}

/// A `Motor` represents a hardware motor. It's given all the necessary configuration information to manage its own position and communication and provides a high-level interface to accomplish tasks.
pub struct Motor {
	pin: u8,
	pulse_width: Duration,
	period: Duration, // 20 ms
	range: (Duration, Duration)
}

impl Motor {

	pub fn new(pin: u8, period: Duration, range: (Duration, Duration)) -> Self {
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