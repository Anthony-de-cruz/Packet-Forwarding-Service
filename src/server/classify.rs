use std::thread;
use std::time::SystemTime;

use crossbeam_channel::{Receiver, Sender, TrySendError};
use tracing::{debug, error};

use crate::classification::model::{Classifier, TrafficType};
use crate::server::monitor::ClassifyMetrics;
use crate::server::route::ConntrackId;

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
/// * `result_tx`: Channel used to report the winning traffic class,
///   or an inference error, back to ingress.
/// * `metrics_tx`: Channel to report metrics to.
pub fn classify_loop(
    classifier: &mut Classifier,
    task_rx: &Receiver<ClassifyTask>,
    result_tx: &Sender<ClassifyResult>,
    metrics_tx: &Sender<ClassifyMetrics>,
) {
    loop {
        // Rx task.
        let task = task_rx
            .recv()
            .expect("Failed to receive classify task. Channel disconnected.");

        // Classify.
        let classification = classifier
            .classify_payload(&task.sample)
            .map(|classification| classification.traffic_type)
            .map_err(|error| -> Box<dyn std::error::Error + Send + Sync> {
                error.to_string().into()
            });

        if let Ok(traffic_type) = &classification {
            let current_thread = thread::current();
            let thread_name = current_thread.name().unwrap_or("classifier").to_owned();
            match metrics_tx.try_send(ClassifyMetrics {
                timestamp: SystemTime::now(),
                thread_name,
                conntrack_id: task.id,
                traffic_type: *traffic_type,
            }) {
                Ok(()) => {}
                Err(TrySendError::Disconnected(_)) => {
                    panic!("Failed to send classify metrics. Channel disconnected.");
                }
                Err(TrySendError::Full(_)) => {
                    // Unexpected, drop metrics.
                    error!("Failed to send classify metrics. Channel full.");
                }
            }
            debug!("Classified 0x{:#010X} -> {}", task.id, traffic_type);
        }

        // Tx result. (Blocking to avoid dropping results, though it is unexpected.)
        result_tx
            .send(ClassifyResult {
                id: task.id,
                classification,
            })
            .expect("Failed to send classify job. Channel disconnected.");
    }
}
