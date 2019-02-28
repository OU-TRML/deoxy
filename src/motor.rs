//! Motor management.

use std::{ops::RangeInclusive, time::Duration};

use crate::{
    actix::*,
    pin::{Error as PinError, Pin},
};

static RETRIES: u8 = 20;

/// A message that can be sent to a motor to change its position.
#[derive(Clone, Copy, Debug)]
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
#[derive(Debug)]
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
    /// The handle to the main loop for this motor (for cancellation).
    main_handle: Option<SpawnHandle>,
    /// The number of consecutive failures followed by retries.
    retries: u8,
}

impl PartialEq for Motor {
    fn eq(&self, other: &Self) -> bool {
        self.pin.number == other.pin.number
    }
}

impl Eq for Motor {}

impl Motor {
    /// Sets the motor's angle in degrees (relative to the closed position).
    ///
    /// ## Panics
    /// This method will panic if `angle` is greater than 180.
    pub fn set_angle(&mut self, angle: u16) {
        assert!(angle <= 180);
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
        log::trace!(
            "Setting motor angle to {} (pulse width: {:?})",
            angle,
            start + offset
        );
        self.pulse_width = start + offset;
    }
    /// Sets the motor to the closed position.
    pub fn close(&mut self) {
        log::trace!("Closing motor.");
        self.set_angle(0)
    }
    /// Sets the motor to the open position (angle of 90º).
    pub fn open(&mut self) {
        log::trace!("Opening motor.");
        self.set_angle(90)
    }
    ///
    /// Constructs a new motor with the given period and signal range on the given pin number, if
    /// possible.
    ///
    /// The motor will be set to the closed position initially.
    pub fn try_new<R>(period: Duration, range: R, pin: u16) -> Result<Self, PinError>
    where
        R: Into<RangeInclusive<Duration>>,
    {
        let pin = Pin::try_new(pin)?;
        let signal_range = range.into();
        Ok(Self {
            period,
            pin,
            pulse_width: *signal_range.start(),
            signal_range,
            main_handle: None,
            retries: 0,
        })
    }
    /// Constructs a new motor with the given period and signal range on the given pin number.
    ///
    /// The motor will be set to the closed position initially.
    ///
    /// ## Panics
    /// This method will panic if opening the pin fails. For a fallible initializer, see
    /// [`Motor::try_new`](#method.try_new).
    pub fn new<R>(period: Duration, range: R, pin: u16) -> Self
    where
        R: Into<RangeInclusive<Duration>>,
    {
        Self::try_new(period, range, pin).expect("Motor construction failed.")
    }
}

/// Warns the user that pin output failed and aborts if appropriate.
// TODO: If we abort, tell the user.
fn retry_or_abort(motor: &mut Motor, context: &mut Context<Motor>) {
    motor.retries += 1;
    log::warn!("Pin {} output failed.", motor.pin.number);
    // Use >= just in case somebody updates this elsewhere for whatever reason.
    if motor.retries >= RETRIES {
        log::error!("Maximum retries ({}) reached; aborting.", RETRIES);
        if let Some(handle) = motor.main_handle.take() {
            context.cancel_future(handle);
        }
    } else {
        log::info!("Will retry next time ({}).", motor.retries);
    }
}

impl Actor for Motor {
    type Context = Context<Self>;
    fn started(&mut self, context: &mut Self::Context) {
        log::trace!("Motor actor started.");
        self.main_handle = context
            .run_interval(self.period, |motor, context| {
                let result = motor.pin.set_high();
                if result.is_err() {
                    retry_or_abort(motor, context);
                }
                context.run_later(motor.pulse_width, |motor, context| {
                    let result = motor.pin.set_low();
                    if result.is_err() {
                        retry_or_abort(motor, context);
                    }
                });
            })
            .into();
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    // This test makes sure the panic in validate_motor_angle isn't from constructing the motor and unwrapping it.
    #[test]
    fn make_fake_motor() {
        let _motor = Motor::try_new(
            Duration::new(2, 0),
            Duration::new(0, 0)..=Duration::new(1, 0),
            1,
        )
        .unwrap();
    }
    #[test]
    #[should_panic]
    fn validate_motor_angle() {
        let mut motor = Motor::try_new(
            Duration::new(2, 0),
            Duration::new(0, 0)..=Duration::new(1, 0),
            1,
        )
        .unwrap();
        motor.set_angle(181);
    }
}
