//! App state management.
use crate::{actix::Addr, Coordinator};

use std::sync::Arc;

/// Contains the coordinator and other required state components.
#[derive(Clone, Debug)]
pub struct State {
    /// The coordinator.
    // We don't need an RwLock because we'll just be sending messages.
    pub coord: Arc<Coordinator>,
    /// The address of the coordinator.
    pub addr: Addr<Coordinator>,
}
