use crate::classification::model::{Classifier, TrafficType};
use crate::server::ingress::ConntrackId;
use crossbeam_channel::{Receiver, Sender, TrySendError};
use std::thread::current;
use tracing::{debug, error, warn};

/// Classification work item sent from ingress to a worker thread.
pub struct ClassifyTask {
    pub(crate) id: ConntrackId,
    pub(crate) sample: Vec<u8>,
}

/// Classification result returned from a worker to ingress.
pub struct ClassifyResult {
    pub(crate) id: ConntrackId,
    pub(crate) classification: Result<TrafficType, Box<dyn std::error::Error + Send + Sync>>,
}

/// Run the background classification worker loop.
///
/// Each worker blocks on the task queue, classifies the buffered flow sample,
/// and publishes the result back to the ingress thread.
///
/// # Arguments
///
/// * `classifier`: Reusable ONNX-backed classifier owned by this worker.
/// * `task_rx`: Channel that delivers byte samples grouped by conntrack flow.
/// * `result_tx`: Channel used to report the winning traffic class, or an
///   inference error, back to ingress.
pub fn classify_loop(
    classifier: &mut Classifier,
    task_rx: &Receiver<ClassifyTask>,
    result_tx: &Sender<ClassifyResult>,
) {
    loop {
        match task_rx.recv() {
            Ok(task) => {
                let classification = classifier
                    .classify_payload(&task.sample)
                    .map(|classification| classification.traffic_type)
                    .map_err(|error| -> Box<dyn std::error::Error + Send + Sync> {
                        error.to_string().into()
                    });

                if let Ok(classification) = &classification {
                    debug!("CLASSIFIED 0x{:X?} -> {}", task.id, classification);
                }

                match result_tx.try_send(ClassifyResult {
                    id: task.id,
                    classification,
                }) {
                    Ok(()) => {}
                    Err(TrySendError::Disconnected(_)) => {
                        error!("Failed to send classify result. Channel disconnected.");
                    }
                    Err(TrySendError::Full(_)) => {
                        warn!("Failed to send classify result. Channel full.");
                    }
                }
            }
            Err(_) => {
                error!(
                    "{} | Failed to receive classify task. Channel disconnected.",
                    current().name().unwrap_or("")
                );
            }
        }
    }
}
