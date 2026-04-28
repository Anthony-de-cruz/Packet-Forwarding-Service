pub mod classify;
pub mod ingress;

use crate::classification::model::TrafficType;

/// Represents a kernel-level conntrack ID for a given flow.
pub type ConntrackId = u32;

pub struct ClassifyTask {
    id: ConntrackId,
    buf: Vec<Vec<u8>>,
}

pub struct ClassifyResult {
    id: ConntrackId,
    classification: Result<TrafficType, Box<dyn std::error::Error + Send + Sync>>,
}
