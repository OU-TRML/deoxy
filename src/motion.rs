//! Components related to motors and their movement.

use std::ops::Range;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[allow(unused_imports)]
use io::{GpioOutputStub, Pin};

pub use angle::Angle;

/// Represents the range of angles that a motor can attain.
#[derive(Clone, Copy, Debug)]
pub enum MotorRange {
    /// Represents a motor range of 180 degrees.
    Full,
    /// Represents a motor range of 90 degrees.
    Half,
    /// Represents a motor range of 45 degrees.
    Quarter,
    /// Represents a custom motor range.
    ///
    /// The associated angle is the upper limit.
    Other(Angle),
}

impl MotorRange {
    /// Returns the maximum angle the motor can attain.
    ///
    /// # Examples
    /// ```
    /// # extern crate deoxy;
    /// # use deoxy::motion::{Angle, MotorRange};
    /// let full = MotorRange::Full;
    /// assert_eq!(full.max_value().measure(), 180.0);
    /// let half = MotorRange::Half;
    /// assert_eq!(half.max_value().measure(), 90.0);
    /// let quarter = MotorRange::Quarter;
    /// assert_eq!(quarter.max_value().measure(), 45.0);
    /// let custom_max = Angle::with_measure(10.0);
    /// let custom = MotorRange::Other(custom_max);
    /// assert_eq!(custom.max_value().measure(), custom_max.measure());
    /// ```
    pub fn max_value(&self) -> Angle {
        match *self {
            MotorRange::Full => Angle::with_measure(180.0),
            MotorRange::Half => Angle::with_measure(90.0),
            MotorRange::Quarter => Angle::with_measure(45.0),
            MotorRange::Other(angle) => angle,
        }
    }

    /// Returns the minimum angle the motor can attain.
    ///
    /// # Examples
    /// ```
    /// # extern crate deoxy;
    /// # use deoxy::motion::{Angle, MotorRange};
    /// let ranges = [MotorRange::Full, MotorRange::Half, MotorRange::Quarter];
    /// for range in &ranges {
    ///     assert_eq!(range.min_value(), Angle::zero());
    /// }
    /// ```
    pub fn min_value(&self) -> Angle {
        Angle::default()
    }

    /// Converts the range into a more primitive version for storage.
    pub fn to_range(&self) -> Range<Angle> {
        self.min_value()..self.max_value() // self.min_value()..=self.max_value()
    }
}

/// Represents a motor mounted on the board.
#[derive(Debug)]
pub struct Motor {
    /// The underlying `Pin` instance which manages the state of the GPIO pin to which the motor is attached.
    pin: Pin,
    /// The current pulse width for the motor signal.
    pulse_width: Arc<Mutex<Duration>>,
    /// The motor's constant, characterisic period.
    period: Duration,
    /// The range of pulse widths this motor supports (used to calculate appropriate widths for neutral and anti-neutral positions).
    // signal_range: Range<Duration>,
    /// The range of angles to which this motor may be rotated
    angle_range: Range<Angle>,
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
    /// The resulting object will have a pulse width of 0 until one is specified or the [`_loop`](../communication/struct.Slave.html#method._loop) method automatically generates one (if applicable).
    pub fn new(pin_number: u16, period: Duration) -> Self {
        Self {
            pin: Pin::new(pin_number),
            period,
            pulse_width: Arc::new(Mutex::new(Duration::new(0, 0))),
            // signal_range,
            angle_range: MotorRange::default().to_range(),
        }
    }

    /// Gets the currently-set pulse width.
    pub fn get_pulse_width(&self) -> Arc<Mutex<Duration>> {
        Arc::clone(&self.pulse_width)
    }

    /// Gets the characteristic period of this motor.
    pub fn get_period(&self) -> Duration {
        self.period
    }

    /// Delegates to [`Pin.do_wave`](../io/struct.Pin.html#method.do_wave)
    pub fn do_wave(&mut self) {
        let pulse_width = self.get_pulse_width();
        let width = pulse_width.lock().unwrap().clone();
        let period = self.get_period();
        drop(pulse_width);
        self.pin.do_wave(width, period).unwrap();
    }
}
