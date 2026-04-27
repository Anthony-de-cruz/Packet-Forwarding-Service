use crate::classification::model::{Classifier, TrafficType};
use crate::server::{ClassifyResult, ClassifyTask};
use crossbeam_channel::{Receiver, RecvError, Sender, TrySendError};
use std::collections::HashMap;
use std::thread::current;
use std::time::Duration;
use tracing::{error, info, warn};

///
///
/// # Arguments
///
/// * `classifier`:
/// * `task_rx`:
/// * `result_tx`:
///
/// returns: ()
pub fn classify_loop(
    classifier: &mut Classifier,
    task_rx: Receiver<ClassifyTask>,
    result_tx: Sender<ClassifyResult>,
) {
    loop {
        match task_rx.recv() {
            Ok(task) => {
                let classification = classify_payloads(classifier, &task.buf);

                if let Ok(classification) = &classification {
                    info!("CLASSIFIED 0x{:X?} -> {}", task.id, classification);
                }

                match result_tx.try_send(ClassifyResult {
                    id: task.id,
                    classification: classification.map_err(|error| error.into()),
                }) {
                    Ok(_) => {}
                    Err(TrySendError::Disconnected(_)) => {
                        error!("Failed to send classify result. Channel disconnected.")
                    }
                    Err(TrySendError::Full(_)) => {
                        warn!("Failed to send classify result. Channel full.")
                    }
                }
            }
            Err(_) => {
                error!(
                    "{} | Failed to receive classify task. Channel disconnected.",
                    current().name().unwrap_or("")
                )
            }
        }
    }
}

fn classify_payloads(
    classifier: &mut Classifier,
    payloads: &[Vec<u8>],
) -> Result<TrafficType, Box<dyn std::error::Error + Send + Sync>> {
    let mut counts = HashMap::<TrafficType, usize>::new();

    for payload in payloads {
        let classification = classifier
            .classify_payload(payload)
            .map_err(|error| -> Box<dyn std::error::Error + Send + Sync> {
                error.to_string().into()
            })?;
        *counts.entry(classification.traffic_type).or_default() += 1;
    }

    counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(traffic_type, _)| traffic_type)
        .ok_or_else(|| "no payload classifications available".into())
}
