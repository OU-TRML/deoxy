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

#[cfg(feature = "use_serde")]
#[cfg_attr(feature = "use_serde", macro_use)]
extern crate serde_derive;

pub use deoxy_core::*;

mod actix {
    pub use actix_web::actix::{
        Actor, Addr, AsyncContext, Context, Handler as Handle, Message as ActixMessage, SpawnHandle,
    };
}

mod comm;
mod config;
mod motor;
pub(crate) mod pin;
mod pump;
#[cfg(feature = "server")]
pub mod server;

pub use self::{
    comm::{Coordinator, Error as CoordError, Message as CoordMessage, State as ExecState},
    config::{Config, MotorConfig, PumpConfig},
    motor::{Message as MotorMessage, Motor},
    pin::Error as PinError,
    pump::{Direction as PumpDirection, Message as PumpMessage, Pump},
};
