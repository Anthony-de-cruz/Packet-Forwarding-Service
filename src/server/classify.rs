use crate::classification::model::{ Classifier, TrafficType};
use crate::server::{ClassifyResult, ClassifyTask};
use crossbeam_channel::{Receiver, RecvTimeoutError, Sender, TrySendError};
use std::thread::current;
use std::time::Duration;

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
        match task_rx.recv_timeout(Duration::from_millis(10)) {
            Ok(task) => {
                println!(
                    "{} | CLASSIFYING {} -> {}",
                    current().name().unwrap_or(""),
                    task.id,
                    TrafficType::GoogleMeet
                );

                match result_tx.try_send(ClassifyResult {
                    id: task.id,
                    classification: Ok(TrafficType::GoogleMeet),
                }) {
                    Ok(_) => {}
                    Err(TrySendError::Disconnected(_)) => {
                        eprintln!("Failed to send classify result. Channel disconnected.")
                    }
                    Err(TrySendError::Full(_)) => {
                        eprintln!("Failed to send classify result. Channel full.")
                    }
                }
            }
            Err(RecvTimeoutError::Disconnected) => {
                eprintln!(
                    "{} | Failed to receive classify task. Channel disconnected.",
                    current().name().unwrap_or("")
                )
            }
            Err(RecvTimeoutError::Timeout) => {}
        }
    }
}
