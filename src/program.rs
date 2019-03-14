//! Utilities for scheduling actions.
use std::time::Duration;

use crate::MotorId;

/// Represents an error encountered while validating a protocol.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    /// Creates a single-step protocol.
    pub fn with_step(step: Step) -> Self {
        Self {
            steps: vec![step],
        }
    }
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
    /// Currently, the last step is the only checked step.
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
    /// Attempts to convert the protocol to a [`program`](struct.Program.html).
    ///
    /// The protocol will first be validated.
    pub fn as_program(&self) -> Result<Program, ValidateError> {
        self.validate()?;
        let mut actions = self
            .steps
            .iter()
            .flat_map(|step| {
                let Step::Perfuse(motor, duration) = step;
                let mut actions = vec![];
                actions.push(Action::Perfuse(*motor));
                actions.push(duration.map(Action::Sleep).unwrap_or(Action::Hail));
                actions.push(Action::Drain);
                actions.into_iter()
            })
            .collect::<Vec<_>>();
        let _ = actions.pop();
        let _ = actions.pop();
        actions.push(Action::Finish);
        assert!(actions.len() > 1);
        if let Action::Perfuse(_) = actions[0] {
            Ok(Program { actions })
        } else {
            // This shouldn't be able to happen, so it's more than user error; it's on us.
            // A panic is appropriate here for this reason.
            panic!("Invalid program detected; no initial perfusion.");
        }
    }
}

/// Represents a specific action to be run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    /// Perfuse with the specified solution until a full volume is reached, then close the valve
    /// and turn off the pump.
    Perfuse(MotorId),
    /// Wait for the specified duration.
    Sleep(Duration),
    /// Wait for the user to continue.
    Hail,
    /// Drain until empty, then turn off the pump.
    Drain,
    /// Finalize the job and notify the user.
    Finish,
}

impl Action {
    /// Whether this action can be performed in isolation from the preceding steps.
    ///
    /// If true, the coordinator will stop *before* this step when stopping early.
    pub fn is_disjoint(self) -> bool {
        match self {
            // These actions come after perfusing, so we can stop after the prior step if need be.
            Action::Sleep(_) | Action::Hail | Action::Finish | Action::Drain => true,
            // Don't stop before perfusing (the sample should not be dry when we're done)
            Action::Perfuse(_) => false,
        }
    }
}

/// A sequence of fine-grained actions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program {
    actions: Vec<Action>,
}

impl IntoIterator for Program {
    type Item = Action;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.actions.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn protocol_as_program() {
        let mut protocol = Protocol {
            steps: vec![Step::Perfuse(0, None), Step::Perfuse(0, None)],
        };
        assert!(protocol.as_program().is_ok());
        protocol
            .steps
            .push(Step::Perfuse(1, Some(Duration::new(2, 0))));
        assert!(protocol.as_program().is_err());
        protocol.steps.clear();
        assert_eq!(protocol.as_program(), Err(ValidateError::Empty));
    }
}
