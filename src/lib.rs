//! Buffer exchange system library.
mod actix {
    pub use actix::{Actor, Context, Handler as Handle, Message as ActixMessage};
}

mod motor;
pub use self::motor::{Message as MotorMessage, Motor};
pub(crate) mod pin;
