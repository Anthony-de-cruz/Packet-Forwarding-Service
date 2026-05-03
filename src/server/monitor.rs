use crate::classification::model::Classification;
use crate::server::route::ConntrackId;
use crossbeam_channel::{Receiver, TryRecvError};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::thread;
use std::time::{Duration, Instant, SystemTime};
use tracing::info;

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
    /// Time of report.
    pub(crate) timestamp: SystemTime,
    /// Worker thread name.
    pub(crate) thread_name: String,
    pub(crate) conntrack_id: ConntrackId,
    pub(crate) classification: Classification,
}

///
const LOOP_SLEEP: Duration = Duration::from_millis(10);
const PRINT_INTERVAL: Duration = Duration::from_secs(1);

///
///
/// # Arguments
///
/// * `ingress_rx`:
/// * `classify_rx`:
#[allow(clippy::cast_precision_loss)]
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

    let mut last_ingress_metrics: Option<IngressMetrics>;
    let mut last_print = Instant::now();

    loop {
        last_ingress_metrics = consume_ingress_metrics(&mut ingress_writer, ingress_rx);
        consume_classify_metrics(&mut classify_writer, classify_rx);

        if last_print.elapsed() > PRINT_INTERVAL {
            match &last_ingress_metrics {
                None => {}
                Some(m) => {
                    let time_delay = SystemTime::now()
                        .duration_since(m.timestamp)
                        .expect("Impossible timestamp.")
                        .as_secs_f64();
                    info!(
                        "Total: {} packets @ {:.2} Gb, Missed: {}",
                        m.packet_total,
                        (m.byte_total as f64 * 8.0) / 1_000_000_000.0,
                        m.unoptimised_packet_total
                    );
                    info!(
                        "Current: {:.2}p/s @ {:.2}Mb/s, Missed {:.2}% @ {:.2}p/s",
                        m.packet_interval as f64 / time_delay,
                        (m.byte_interval as f64 / time_delay) * 8.0 / 1_000_000.0,
                        m.unoptimised_packet_interval as f64 / m.packet_interval as f64 * 100.0,
                        m.unoptimised_packet_interval as f64 / time_delay,
                    );
                }
            }
            last_print = Instant::now();
            thread::sleep(LOOP_SLEEP);
        }
    }

    // ingress_writer.flush().unwrap();
    // classify_writer.flush().unwrap();
}

fn consume_ingress_metrics(
    writer: &mut BufWriter<File>,
    ingress_rx: &Receiver<IngressMetrics>,
) -> Option<IngressMetrics> {
    let mut last_metrics: Option<IngressMetrics> = None;
    loop {
        match ingress_rx.try_recv() {
            Ok(m) => {
                writeln!(
                    writer,
                    "{:?}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                    m.timestamp,
                    m.flow_count,
                    m.classify_backpressure,
                    m.packet_total,
                    m.byte_total,
                    m.packet_interval,
                    m.byte_interval,
                    m.unoptimised_packet_total,
                    m.unoptimised_byte_total,
                    m.unoptimised_byte_interval
                )
                .expect("Failed to write ingress metrics to disk.");
                last_metrics = Some(m);
            }
            Err(TryRecvError::Disconnected) => {
                panic!("Failed to receive ingress metrics. Channel disconnected.");
            }
            Err(TryRecvError::Empty) => {
                return last_metrics;
            }
        }
    }
}

fn consume_classify_metrics(writer: &mut BufWriter<File>, classify_rx: &Receiver<ClassifyMetrics>) {
    loop {
        match classify_rx.try_recv() {
            Ok(m) => {
                writeln!(
                    writer,
                    "{:?}|{}|{}|{}",
                    m.timestamp, m.thread_name, m.conntrack_id, m.classification.traffic_type,
                )
                .expect("Failed to write classify metrics to disk.");
                //info!("{} | {} -> {}", m.thread_name);
            }
            Err(TryRecvError::Disconnected) => {
                panic!("Failed to receive classify metrics. Channel disconnected.");
            }
            Err(TryRecvError::Empty) => {
                break;
            }
        }
    }
}
