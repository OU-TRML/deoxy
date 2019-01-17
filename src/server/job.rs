use super::state::State as AppState;
use crate::{comm::State, Action, MotorId, Program};
use actix_web::{HttpRequest, Responder};
use uuid::Uuid;

/// Represents a (buffer-exchange) job to be run.
pub struct Job {
    id: Uuid,
    state: State,
    program: Option<Program>,
    remaining: Vec<Action>,
    buffer: Option<MotorId>,
}

/// The current status of the device.
#[allow(clippy::needless_pass_by_value)]
pub fn status(req: HttpRequest<AppState>) -> impl Responder {
    let coord = &req.state().coord;
    if let Some(uuid) = coord.state.uuid {
        let state = coord.status();
        let program = coord.state.program.clone();
        let remaining = coord.state.remaining.clone();
        let buffer = coord.state.buffer;
        let _job = Job {
            id: uuid,
            state,
            program,
            remaining,
            buffer,
        };
    }
    ""
}

/// Creates and starts a new job if the system is ready.
#[allow(clippy::needless_pass_by_value)]
pub fn start(req: HttpRequest<AppState>) -> impl Responder {
    let coord = &req.state().coord;
    if let Some(_uuid) = coord.state.uuid {
        // Error
    } else {
        // Start the job
    }
    ""
}
