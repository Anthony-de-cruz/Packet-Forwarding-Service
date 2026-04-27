pub mod ingress;
pub mod classify;

use crate::classification::model::TrafficType;

pub type ConntrackId = u32;

pub struct ClassifyTask {
    id: ConntrackId,
    buf: Vec<Vec<u8>>,
}

pub struct ClassifyResult {
    id: ConntrackId,
    classification: Result<TrafficType, Box<dyn std::error::Error + Send + Sync>>,
}
