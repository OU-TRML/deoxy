//! App state management.
use crate::Coordinator;

use std::sync::Arc;

/// Contains the coordinator and other required state components.
#[derive(Clone, Debug)]
pub struct State {
    pub coord: Arc<Coordinator>,
}
