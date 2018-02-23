use std::time::Duration;
use std::ops::Range;
use std::sync::{Arc, Mutex};

#[allow(unused_imports)]
use io::{Pin, GpioOutputStub};

type Angle = u16;

pub enum MotorRange {
	/// Represents a motor range of 180 degrees.
	Full,
	/// Represents a motor range of 90 degrees.
	Half,
	/// Represents a motor range of 45 degrees.
	Quarter,
	/// Represents a custom motor range (in degrees).
	Other(Angle)
}

impl MotorRange {

	pub fn max_value(&self) -> Angle {
		match *self {
			MotorRange::Full => 180,
			MotorRange::Half => 90,
			MotorRange::Quarter => 45,
			MotorRange::Other(angle) => angle
		}
	}

	pub fn min_value(&self) -> Angle {
		0
	}

	pub fn to_range(&self) -> Range<Angle> {
		self.min_value()..(self.max_value() + 1) // self.min_value()..=self.max_value()
	}

}

/// Represents a motor mounted on the board.
pub struct Motor {
	/// The underlying `Pin` instance which manages the state of the GPIO pin to which the motor is attached.
	pin: Arc<Mutex<Pin>>,
	/// The current (cached) pulse width for the motor signal.
	pulse_width: Duration,
	/// The motor's constant, characterisic period.
	period: Duration,
	/// The range of pulse widths this motor supports (used to calculate appropriate widths for neutral and anti-neutral positions).
	signal_range: Range<Duration>,
	/// The range of angles to which this motor may be rotated
	angle_range: Range<Angle>
}

impl Default for MotorRange {
	fn default() -> Self {
		MotorRange::Full
	}
}

// TODO: From methods
impl Motor {

	/// Constructs a new `Motor` with the given properties.
	/// # Notes
	/// You are discouraged from using this method outside of testing environments; you should instead use appropriate `From` methods (see below).
	///
	/// This method will also instantiate an underlying `Pin`.
	///
	/// The resulting object will have a pulse width of 0 until one is specified or the [`_loop`](#method._loop) method automatically generates one (if applicable).
	pub fn new(pin_number: u16, period: Duration, signal_range: Range<Duration>) -> Self {
		Self {
			pin: Arc::new(Mutex::new(Pin::new(pin_number))),
			period,
			pulse_width: Duration::new(0, 0),
			signal_range,
			angle_range: MotorRange::default().to_range()
		}
	}

	/// Sets the motor to the neutral position.
	pub fn set_neutral(&mut self) {
		self.pulse_width = (self.signal_range.start + self.signal_range.end) / 2;
	}

	/// Sets the motor angle (in degrees, unfortunately).
	/// # Errors
	/// If the given `angle` doesn't lie within [`angle_range`](#field.angle_range), this method returns Err(()) and nothing happens.
	pub fn set_angle(&mut self, angle: f32) -> Result<(), ()> {
		if angle < self.angle_range.start as f32 || angle > self.angle_range.end as f32 {
			Err(())
		} else {
			let ratio = (self.signal_range.end - self.signal_range.start) / ((self.angle_range.end - self.angle_range.start) as u32);
			let (seconds, nanoseconds) = (ratio.as_secs(), ratio.subsec_nanos());
			self.pulse_width = Duration::new((seconds as f32 * angle).round() as u64, (nanoseconds as f32 * angle).round() as u32);
			Ok(())
		}
	}

	/// Sets the motor to the "zero" position.
	pub fn set_orthogonal(&mut self) {
		let _ = self.set_angle(0.0).unwrap();
	}

	/// Gets the currently-set pulse width.
	pub fn get_pulse_width(&self) -> Duration {
		self.pulse_width
	}

	/// Gets the characteristic period of this motor.
	pub fn get_period(&self) -> Duration {
		self.period
	}

	/// Delegates to [`Pin.do_wave`](struct.Pin.html#method.do_wave)
	pub fn do_wave(&mut self) {
		let _ = self.pin.lock().unwrap().do_wave(self.get_pulse_width(), self.get_period()).unwrap();
	}

}