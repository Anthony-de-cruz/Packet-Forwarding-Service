mod classification;
mod server;

use std::thread;

use crossbeam_channel::{Receiver, Sender, bounded};
use nfq::Queue;
use time::format_description::parse;
use tracing::info;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::time::UtcTime;

use crate::classification::model::Classifier;
use crate::server::route::ingress_loop;
use crate::server::classify::{ClassifyResult, ClassifyTask, classify_loop};
use crate::server::monitor::{monitor_loop, ClassifyMetrics, IngressMetrics};

/// Path to model.
const MODEL_PATH: &str = "Packet-Classifier/out/model.onnx";
/// Netfilter queue number.
const NF_QUEUE_NUM: u16 = 10;
/// Size of cross-thread channels.
const CHANNEL_SIZE: usize = 1024;

fn open_classifier() -> Result<Classifier, Box<dyn std::error::Error>> {
    info!("Loading traffic classifier from {MODEL_PATH}...");
    let classifier = Classifier::from_file(MODEL_PATH)?;
    info!("Traffic classifier ready.");
    Ok(classifier)
}

fn open_nfqueue() -> std::io::Result<Queue> {
    info!("Opening Netfilter queue {NF_QUEUE_NUM}...");
    let mut queue = Queue::open()?;
    queue.bind(NF_QUEUE_NUM)?;
    queue.set_recv_conntrack(NF_QUEUE_NUM, true)?;
    info!("Netfilter queue {NF_QUEUE_NUM} ready.");
    Ok(queue)
}

fn start_classify_workers(
    worker_count: usize,
    task_rx: &Receiver<ClassifyTask>,
    result_tx: &Sender<ClassifyResult>,
    metrics_tx: &Sender<ClassifyMetrics>,
) {
    for worker_id in 0..worker_count {
        let rx = task_rx.clone();
        let result_tx = result_tx.clone();
        let metrics_tx = metrics_tx.clone();
        let mut classifier = open_classifier().unwrap();

        thread::Builder::new()
            .name(format!("classifier-{worker_id}"))
            .spawn(move || classify_loop(&mut classifier, &rx, &result_tx, &metrics_tx))
            .expect("Failed to spawn thread.");
    }
}

fn start_monitor_worker(
    ingress_metrics_rx: &Receiver<IngressMetrics>,
    classify_metrics_rx: &Receiver<ClassifyMetrics>
) {
    let ingress = ingress_metrics_rx.clone();
    let classify = classify_metrics_rx.clone();
    thread::Builder::new()
        .name(String::from("monitor"))
        .spawn(move || monitor_loop(&ingress, &classify))
        .expect("Failed to spawn thread.");
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

    // Create cross-thread channels.
    let (task_tx, task_rx) = bounded::<ClassifyTask>(CHANNEL_SIZE);
    let (result_tx, result_rx) = bounded::<ClassifyResult>(CHANNEL_SIZE);
    let (ingress_metrics_tx, ingress_metrics_rx) = bounded::<IngressMetrics>(CHANNEL_SIZE);
    let (classify_metrics_tx, classify_metrics_rx) = bounded::<ClassifyMetrics>(CHANNEL_SIZE);

    start_classify_workers(3, &task_rx, &result_tx, &classify_metrics_tx);
    start_monitor_worker(&ingress_metrics_rx, &classify_metrics_rx);
    ingress_loop(&mut open_nfqueue()?, &task_tx, &result_rx, &ingress_metrics_tx);
    Ok(())
}
