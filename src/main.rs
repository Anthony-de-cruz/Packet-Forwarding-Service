mod classification;
mod server;

use crossbeam_channel::{Receiver, Sender, bounded, unbounded};
use nfq::Queue;

use crate::classification::model::Classifier;
use crate::server::classify::classify_loop;
use crate::server::ingress::{ClassifyResult, ClassifyTask, ingress_loop};
use std::thread;

const MODEL_PATH: &str = "Packet-Classifier/out/model.onnx";
const QUEUE_NUM: u16 = 10;

fn open_classifier() -> Result<Classifier, Box<dyn std::error::Error>> {
    println!("Loading traffic classifier from {MODEL_PATH}...");
    let classifier = Classifier::from_file(MODEL_PATH)?;
    println!("Traffic classifier ready.");
    Ok(classifier)
}

fn open_nfqueue() -> std::io::Result<Queue> {
    println!("Opening netfilter queue {QUEUE_NUM}...");
    let mut queue = Queue::open()?;
    queue.bind(QUEUE_NUM)?;
    queue.set_recv_conntrack(QUEUE_NUM, true)?;
    println!("Netfilter queue {QUEUE_NUM} ready.");
    Ok(queue)
}

fn start_classify_workers(
    worker_count: usize,
    task_rx: Receiver<ClassifyTask>,
    result_tx: Sender<ClassifyResult>,
) {
    for worker_id in 0..worker_count {
        let rx = task_rx.clone();
        let tx = result_tx.clone();
        let mut classifier = open_classifier().unwrap();

        thread::Builder::new()
            .name(format!("classifier-{worker_id}"))
            .spawn(move || classify_loop(&mut classifier, rx, tx))
            .expect("Failed to spawn thread.");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (task_tx, task_rx) = bounded::<ClassifyTask>(1024);
    let (result_tx, result_rx) = unbounded::<ClassifyResult>();

    start_classify_workers(1, task_rx, result_tx);
    ingress_loop(&mut open_nfqueue()?, task_tx, result_rx);
    Ok(())
}
