//! Components related to motors, pumps, and their movement.

use std::ops::Range;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[allow(unused_imports)]
use io::{GpioOutputStub, Pin};

pub use angle::Angle;

use config::PumpSpec;

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

/// Represents a direction for the master pump (either forward, backward, or off).
///
/// ### Notes
/// This enum does not contain information about pump speed.
#[derive(Clone, Copy, Debug)]
pub enum PumpDirection {
    /// The "forward" direction is the direction in which the sample is actively perfused.
    Forward,
    /// The "backward" direction is the direction in which the sample is drained.
    Backward,
    /// The "off" state means that the pump receives no power.
    Off,
}

/// Represents a big 'ol pump that enables perfusion and drainage.
///
/// ### Notes
/// In the canonical pump circuit, the on/off relay is normally open (off).
/// The positive/negative relay is also normally open (positive).
/// Powering the on/off relay causes the pump to run, either forward (± off) or backward (± on).
#[derive(Debug)]
pub struct Pump {
    /// The pin corresponding to the on/off relay.
    toggle_pin: Pin,
    /// The pin corresponding to the positive/negative relay.
    invert_pin: Pin,
    /// The current direction of the pump.
    direction: PumpDirection,
}

impl Pump {
    /// Creates a new pump on the pins specified and sets it to closed.
    pub fn new(toggle_pin_number: u16, invert_pin_number: u16) -> Self {
        Self {
            toggle_pin: Pin::new(toggle_pin_number),
            invert_pin: Pin::new(invert_pin_number),
            direction: PumpDirection::Off,
        }
    }
    /// Returns the pump's current direction.
    pub fn get_direction(&self) -> PumpDirection {
        self.direction
    }
    /// Turns off the pump and opens both relays.
    pub fn close(&mut self) {
        self.toggle_pin.set_low().unwrap();
        self.invert_pin.set_low().unwrap();
        self.direction = PumpDirection::Off;
    }
    /// Tells the pump to perfuse (run forward).
    pub fn perfuse(&mut self) {
        self.invert_pin.set_low().unwrap();
        self.toggle_pin.set_high().unwrap();
    }
    /// Tells the pump to drain (run backward).
    pub fn drain(&mut self) {
        self.invert_pin.set_high().unwrap();
        self.toggle_pin.set_high().unwrap();
    }
}

impl<'a> From<&'a PumpSpec> for Pump {
    fn from(spec: &'a PumpSpec) -> Self {
        Self::new(spec.pins[0], spec.pins[1]) // TODO: Merge H-bridge commit.
    }
}
