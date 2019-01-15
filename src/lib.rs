//! Buffer exchange system library.
#![forbid(unsafe_code)]
#![deny(
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces
)]
#![deny(clippy::use_self, clippy::wildcard_dependencies)]
#![warn(unused_qualifications)]
#![warn(
    clippy::print_stdout,
    clippy::pub_enum_variant_names,
    clippy::used_underscore_binding,
    clippy::wrong_self_convention,
    clippy::wrong_pub_self_convention
)]

/// Used to uniquely identify motors/valves.
pub type MotorId = usize;

#[cfg(feature = "full")]
mod actix {
    pub use actix_web::actix::{Actor, Context, Handler as Handle, Message as ActixMessage};
}

#[cfg(feature = "full")]
mod comm;
#[cfg(feature = "full")]
mod motor;
#[cfg(feature = "full")]
pub(crate) mod pin;
#[cfg(feature = "full")]
mod pump;

#[cfg(feature = "full")]
pub use self::{
    comm::{Coordinator, Error as CoordError, State as ExecState},
    motor::{Message as MotorMessage, Motor},
    pin::Error as PinError,
    pump::{Direction as PumpDirection, Message as PumpMessage, Pump},
};

mod program;
pub use self::program::{Action, Program, Protocol, Step, ValidateError as ValidateProtocolError};
