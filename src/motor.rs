//! Motor management.

use std::{ops::RangeInclusive, time::Duration};

use crate::{actix::*, pin::Pin};

/// A message that can be sent to a motor to change its position.
pub enum Message {
    /// Requests that the motor be set to the closed position.
    Close,
    /// Requests that the motor be set to the open position.
    Open,
}

impl ActixMessage for Message {
    type Result = ();
}

/// A motor connected to the syringe manifold.
///
/// Moving a motor (physically) will cause the control knob to rotate.
#[derive(Debug, Eq, PartialEq)]
pub struct Motor {
    /// The characteristic period of the motor.
    period: Duration,
    /// The output pin controlling the physical motor.
    pin: Pin,
    /// The range of acceptable signal lengths.
    ///
    /// The motor is assumed to have 180º of motion, meaning the minimum and signals should
    /// correspond to antiparallel positions.
    ///
    /// The closed position is assumed to be 0º; the open position is at 90º.
    signal_range: RangeInclusive<Duration>,
    /// The duration for which the signal should be high in each period.
    ///
    /// Changing this property will change the position of the motor.
    pulse_width: Duration,
}

impl Motor {
    /// Sets the motor's angle in degrees (relative to the closed position).
    ///
    /// ## Panics
    /// This method will panic if `angle` is less than 0 or greater than 180.
    pub fn set_angle(&mut self, angle: u16) {
        let (start, end) = (self.signal_range.start(), self.signal_range.end());
        // Dereference, since auto-deref doesn't seem to work for std::ops::Sub?
        let (start, end) = (*start, *end);
        let delta = end - start;
        // Assume a range of motion of 180º.
        let range = 180;
        // Calculate the change in signal per unit angle (dT/dθ).
        let step = delta / range;
        // Multiply the step by the desired angle to get the offset from the baseline (∆T).
        let offset = step * angle.into();
        self.pulse_width = start + offset;
    }
    /// Sets the motor to the closed position.
    pub fn close(&mut self) {
        self.set_angle(0)
    }
    /// Sets the motor to the open position (angle of 90º).
    pub fn open(&mut self) {
        self.set_angle(90)
    }
    /// Constructs a new motor with the given period and signal range on the given pin number, if
    /// possible.
    ///
    /// The motor will be set to the closed position initially.
    pub fn new<R>(period: Duration, range: R, pin: u8) -> Self
    where
        R: Into<RangeInclusive<Duration>>,
    {
        let pin: Pin = unimplemented!();
        let signal_range = range.into();
        Self {
            period,
            pin,
            pulse_width: *signal_range.start(),
            signal_range,
        }
    }
}

impl Actor for Motor {
    type Context = Context<Self>;
}

impl Handle<Message> for Motor {
    type Result = ();
    fn handle(&mut self, message: Message, _context: &mut Self::Context) -> Self::Result {
        match message {
            Message::Open => self.open(),
            Message::Close => self.close(),
        }
    }
}
