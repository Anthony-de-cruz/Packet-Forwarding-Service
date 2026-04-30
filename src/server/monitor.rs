use crate::classification::model::Classification;
use crate::server::ingress::ConntrackId;
use crossbeam_channel::{Receiver, TryRecvError};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Represents metrics to be collected from the ingress thread.
pub struct IngressMetrics {
    /// Time of report.
    pub(crate) timestamp: SystemTime,
    /// Number of tracked flows.
    pub(crate) flow_count: usize,
    /// Number of classify tasks waiting to be done.
    pub(crate) classify_backpressure: usize,
    pub(crate) packet_total: usize,
    pub(crate) byte_total: usize,
    pub(crate) packet_interval: usize,
    pub(crate) byte_interval: usize,
    pub(crate) unoptimised_packet_total: usize,
    pub(crate) unoptimised_byte_total: usize,
    pub(crate) unoptimised_packet_interval: usize,
    pub(crate) unoptimised_byte_interval: usize,
}

/// Represents metrics to be collected from each classify thread.
pub struct ClassifyMetrics {
    thread_name: String,
    conntrack_id: ConntrackId,
    classification: Classification,
}

const LOOP_SLEEP: Duration = Duration::from_millis(100);

pub fn monitor_loop(
    ingress_rx: &Receiver<IngressMetrics>,
    classify_rx: &Receiver<ClassifyMetrics>,
) {
    let ingress_log = OpenOptions::new()
        .create(true)
        .append(true)
        .open("ingress.log")
        .unwrap();
    let mut ingress_writer = BufWriter::new(ingress_log);

    let classify_log = OpenOptions::new()
        .create(true)
        .append(true)
        .open("classify.log")
        .unwrap();
    let mut classify_writer = BufWriter::new(classify_log);

    loop {
        consume_ingress_metrics(&mut ingress_writer, ingress_rx);
        consume_classify_metrics(&mut classify_writer, classify_rx);

        thread::sleep(LOOP_SLEEP);
    }

    ingress_writer.flush().unwrap();
    classify_writer.flush().unwrap();
}

fn consume_ingress_metrics(writer: &mut BufWriter<File>, ingress_rx: &Receiver<IngressMetrics>) {
    loop {
        match ingress_rx.try_recv() {
            Ok(m) => {
                // writeln!(
                //     writer,
                //     "{:?}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                //     m.timestamp,
                //     m.flow_count,
                //     m.classify_backpressure,
                //     m.packet_total,
                //     m.byte_total,
                //     m.packet_interval,
                //     m.byte_interval,
                //     m.unop
                // )
                // .unwrap();
            }
            Err(TryRecvError::Disconnected) => {
                panic!("BRUH");
            }
            Err(TryRecvError::Empty) => {
                break;
            }
        }
    }
}

fn consume_classify_metrics(
    classify_writer: &mut BufWriter<File>,
    classify_rx: &Receiver<ClassifyMetrics>,
) {
    loop {
        match classify_rx.try_recv() {
            Ok(classify_metrics) => {}
            Err(TryRecvError::Disconnected) => {
                panic!("BRUH2");
            }
            Err(TryRecvError::Empty) => {
                break;
            }
        }
    }
}
