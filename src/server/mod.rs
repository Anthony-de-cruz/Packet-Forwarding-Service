pub mod classify;
pub mod ingress;

use crate::classification::model::TrafficType;

/// Represents a kernel-level conntrack ID for a given flow.
pub type ConntrackId = u32;

/// Classification work item sent from ingress to a worker thread.
pub struct ClassifyTask {
    id: ConntrackId,
    buf: Vec<Vec<u8>>,
}

/// Classification result returned from a worker to ingress.
pub struct ClassifyResult {
    id: ConntrackId,
    classification: Result<TrafficType, Box<dyn std::error::Error + Send + Sync>>,
}
