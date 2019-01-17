//! Buffer exchange system core library.
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

mod program;
pub use self::program::{Action, Program, Protocol, Step, ValidateError as ValidateProtocolError};

#[cfg(feature = "use_serde")]
pub extern crate serde;
#[cfg(feature = "use_serde")]
#[cfg_attr(feature = "use_serde", macro_use)]
pub extern crate serde_derive;
