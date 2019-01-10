//! Utilities for scheduling actions.
use std::time::Duration;

use crate::MotorId;

/// Represents an error encountered while validating a protocol.
#[derive(Clone, Copy, Debug)]
pub enum ValidateError {
    /// The protocol is empty and so cannot be valid.
    Empty,
    /// The last step is not an indefinite perfusion.
    Last(Step),
}

/// Represents a high-level step to be taken in a protocol.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Step {
    /// The specified motor should fully perfuse the tissue for the given duration (or until
    /// otherwise instructed if `None`).
    ///
    /// Currently, perfusion is the only valid action, but this may change in the future.
    Perfuse(MotorId, Option<Duration>),
}

/// A high-level description of a series of actions to be taken.
///
/// This is what the end user will feed in (by way of a form).
#[derive(Clone, Debug)]
pub struct Protocol {
    steps: Vec<Step>,
}

impl Protocol {
    /// Ensures the validity of the protocol.
    ///
    /// This method is called automatically during the conversion to `Program`, but it can also be
    /// useful to call it manually.
    ///
    /// ## Details
    /// All protocols should end with a perfusion (in the final solution, usually water) for an
    /// unspecified duration (i.e. a bath). If this is not the case, something's wrong with the
    /// protocol and we should refuse to run it.
    ///
    /// Currently, the last step is the only checked step. Future versions will include checks for
    /// intermediate indefinite perfusions that are not followed by prompts.
    pub fn validate(&self) -> Result<(), ValidateError> {
        if let Some(last) = self.steps.last() {
            match last {
                Step::Perfuse(_, duration) => {
                    if duration.is_none() {
                        Ok(())
                    } else {
                        Err(ValidateError::Last(*last))
                    }
                }
            }
        } else {
            Err(ValidateError::Empty)
        }
    }
}
