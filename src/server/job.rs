use super::state::State as AppState;
use crate::{
    comm::{Message, State},
    Action, MotorId, Program, Protocol,
};
use actix_web::{
    http::header, AsyncResponder, HttpMessage, HttpRequest, HttpResponse, Json, Path, Responder,
    ResponseError,
};
use futures::prelude::*;
use uuid::Uuid;

use std::fmt;

/// Represents a (buffer-exchange) job to be run.
#[derive(Deserialize, Serialize)]
pub struct Job {
    id: Uuid,
    state: State,
    // TODO: Protocol, not program
    program: Option<Program>,
    remaining: Vec<Action>,
    buffer: Option<MotorId>,
}

/// Job request error type.
#[derive(Debug)]
pub enum Error {
    Coordinator(crate::comm::Error),
    Json(actix_web::error::JsonPayloadError),
    Mailbox(actix_web::actix::MailboxError),
    InvalidUuid,
    IncorrectUuid,
}

impl From<crate::comm::Error> for Error {
    fn from(err: crate::comm::Error) -> Self {
        Error::Coordinator(err)
    }
}

impl From<actix_web::error::JsonPayloadError> for Error {
    fn from(err: actix_web::error::JsonPayloadError) -> Self {
        Error::Json(err)
    }
}

impl From<actix_web::actix::MailboxError> for Error {
    fn from(err: actix_web::actix::MailboxError) -> Self {
        Error::Mailbox(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Coordinator(_e) => unimplemented!(),
            Error::Json(e) => e.fmt(f),
            Error::Mailbox(e) => e.fmt(f),
            Error::InvalidUuid => write!(f, "Invalid UUID"),
            Error::IncorrectUuid => write!(f, "Specified job is no longer active."),
        }
    }
}

impl std::error::Error for Error {}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        unimplemented!()
    }
}

/// The current status of the device.
// TODO: HEAD support
#[allow(clippy::needless_pass_by_value)]
pub fn status(req: HttpRequest<AppState>) -> Json<Option<Job>> {
    let coord = &req.state().coord;
    if let Some(uuid) = coord.state.uuid {
        let state = coord.status();
        let program = coord.state.program.clone();
        let remaining = coord.state.remaining.clone();
        let buffer = coord.state.buffer;
        let job = Job {
            id: uuid,
            state,
            program,
            remaining,
            buffer,
        };
        Json(Some(job))
    } else {
        Json(None)
    }
}

/// Creates and starts a new job if the system is ready.
#[allow(clippy::needless_pass_by_value)]
pub fn start(req: HttpRequest<AppState>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    req.json()
        .from_err()
        .and_then(move |proto: Protocol| {
            let coord = &req.state().coord;
            if !coord.is_stopped() {
                Err(Error::from(crate::comm::Error::Busy))
            } else {
                let addr = &req.state().addr;
                let id = Uuid::new_v4();
                let result = addr
                    .send(Message::Start(proto, Some(id)))
                    .map(move |_| {
                        HttpResponse::Created()
                            .header(self::header::LOCATION, format!("{}", id))
                            .finish()
                    })
                    .from_err();
                Ok(result)
            }
        })
        .flatten()
        .responder()
}

// TODO: Custom extractor for UUIDs
// TODO: Custom middleware enforcing correct UUIDs
#[allow(clippy::needless_pass_by_value)]
fn message_for_job(
    message: Message,
    path: Path<String>,
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let coord = &req.state().coord;
    let current = coord.state.uuid;
    let uuid = Uuid::parse_str(&path.into_inner()).ok();
    (if uuid.is_some() {
        if current == uuid {
            let addr = &req.state().addr;
            Ok(addr.send(message))
        } else {
            Err(Error::IncorrectUuid)
        }
    } else {
        Err(Error::InvalidUuid)
    })
    .into_future()
    .map(|_| HttpResponse::NoContent().finish())
    .responder()
}

/// Resumes a job that is waiting for user confirmation.
///
/// We require the user to specify the job in question so that there can be no ambiguity.
#[allow(clippy::needless_pass_by_value)]
pub fn resume(
    path: Path<String>,
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    message_for_job(Message::Continue, path, req)
}

/// Immediately stops the running job.
#[allow(clippy::needless_pass_by_value)]
pub fn halt(
    path: Path<String>,
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    message_for_job(Message::Halt, path, req)
}
