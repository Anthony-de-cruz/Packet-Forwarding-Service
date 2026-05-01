mod classification;
mod server;

use crossbeam_channel::{Receiver, Sender, bounded, unbounded};
use nfq::Queue;
use time::format_description::parse;
use tracing::info;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::time::UtcTime;

use crate::classification::model::Classifier;
use crate::server::classify::{ClassifyResult, ClassifyTask, classify_loop};
use crate::server::ingress::ingress_loop;
use std::thread;

const MODEL_PATH: &str = "Packet-Classifier/out/model.onnx";
const QUEUE_NUM: u16 = 10;

fn open_classifier() -> Result<Classifier, Box<dyn std::error::Error>> {
    info!("Loading traffic classifier from {MODEL_PATH}...");
    let classifier = Classifier::from_file(MODEL_PATH)?;
    info!("Traffic classifier ready.");
    Ok(classifier)
}

fn open_nfqueue() -> std::io::Result<Queue> {
    info!("Opening Netfilter queue {QUEUE_NUM}...");
    let mut queue = Queue::open()?;
    queue.bind(QUEUE_NUM)?;
    queue.set_recv_conntrack(QUEUE_NUM, true)?;
    info!("Netfilter queue {QUEUE_NUM} ready.");
    Ok(queue)
}

fn start_classify_workers(
    worker_count: usize,
    task_rx: &Receiver<ClassifyTask>,
    result_tx: &Sender<ClassifyResult>,
) {
    for worker_id in 0..worker_count {
        let rx = task_rx.clone();
        let tx = result_tx.clone();
        let mut classifier = open_classifier().unwrap();

        thread::Builder::new()
            .name(format!("classifier-{worker_id}"))
            .spawn(move || classify_loop(&mut classifier, &rx, &tx))
            .expect("Failed to spawn thread.");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up logging.
    let log_time_format = parse("[hour]:[minute]:[second]")?;
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("packet_forwarding_service=info")),
        )
        .with_timer(UtcTime::new(log_time_format))
        .with_thread_names(true)
        .with_target(false)
        .compact()
        .init();

    let (task_tx, task_rx) = bounded::<ClassifyTask>(1024);
    let (result_tx, result_rx) = unbounded::<ClassifyResult>();

    start_classify_workers(3, &task_rx, &result_tx);
    ingress_loop(&mut open_nfqueue()?, &task_tx, &result_rx);
    Ok(())
}
