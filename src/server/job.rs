use super::state::State as AppState;
use crate::{
    comm::{Message, State},
    Action, MotorId, Program, Protocol,
};
use actix_web::{
    http::header, AsyncResponder, FromRequest, HttpMessage, HttpRequest, HttpResponse, Json, Path,
    Responder, ResponseError,
};
use futures::prelude::*;
use uuid::Uuid;

use std::{fmt, ops::Deref};

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
    ActixWeb(actix_web::Error),
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

impl From<actix_web::Error> for Error {
    fn from(err: actix_web::Error) -> Self {
        Error::ActixWeb(err)
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
            Error::ActixWeb(e) => e.fmt(f),
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

/// Wrapper type around `Uuid`.
///
/// This struct implements some convenience methods and helps us avoid the orphan rules.
pub struct UUID(Uuid);

impl UUID {
    /// Whether this is the UUID of the currently-running job.
    pub fn is_current(&self, state: &AppState) -> bool {
        let current = state.coord.state.uuid;
        current == Some(**self)
    }
}

impl Deref for UUID {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Uuid> for UUID {
    fn from(uuid: Uuid) -> Self {
        UUID(uuid)
    }
}

impl<S> FromRequest<S> for UUID {
    type Config = ();
    type Result = Result<Self, Error>;
    fn from_request(req: &HttpRequest<S>, _: &Self::Config) -> Self::Result {
        let path = Path::<String>::extract(req).map_err(Error::from);
        path.and_then(|path| Uuid::parse_str(&path.into_inner()).map_err(|_| Error::InvalidUuid))
            .map(UUID)
    }
}

#[allow(clippy::needless_pass_by_value)]
fn message_uuid(
    message: Message,
    uuid: UUID,
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let state = &req.state();
    (if uuid.is_current(&state) {
        let addr = &state.addr;
        Ok(addr.send(message))
    } else {
        Err(Error::IncorrectUuid)
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
    uuid: UUID,
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    message_uuid(Message::Continue, uuid, req)
}

/// Immediately stops the running job.
#[allow(clippy::needless_pass_by_value)]
pub fn halt(
    uuid: UUID,
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    message_uuid(Message::Halt, uuid, req)
}
