use std::fs::{File, OpenOptions, create_dir_all};
use std::io::{BufWriter, Write};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crossbeam_channel::{Receiver, TryRecvError};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tracing::info;

use crate::classification::model::TrafficType;
use crate::server::route::ConntrackId;

const LOOP_SLEEP: Duration = Duration::from_millis(10);
const PRINT_INTERVAL: Duration = Duration::from_secs(1);

/// Represents metrics to be collected from the ingress thread.
pub struct IngressMetrics {
    /// Time of report.
    pub(crate) timestamp: SystemTime,
    /// Length of the sampling interval.
    pub(crate) interval: Duration,
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
    pub(crate) traffic_type: TrafficType,
}

/// Infinite loop to receive metrics and write to disk periodically.
///
/// # Arguments
///
/// * `ingress_rx`: Channel to receive ingress metrics.
/// * `classify_rx`: Channel to receive classify metrics.
#[allow(clippy::cast_precision_loss)]
pub fn monitor_loop(
    ingress_rx: &Receiver<IngressMetrics>,
    classify_rx: &Receiver<ClassifyMetrics>,
) {
    create_dir_all("./out").expect("Failed to create ./out");
    let mut ingress_writer = open_csv_writer(
        "out/ingress.csv",
        "timestamp_utc,timestamp_unix_ms,interval_secs,flow_count,classify_backpressure,packet_total,byte_total,packet_interval,byte_interval,unoptimised_packet_total,unoptimised_byte_total,unoptimised_packet_interval,unoptimised_byte_interval",
    );

    let mut classify_writer = open_csv_writer(
        "out/classify.csv",
        "timestamp_utc,timestamp_unix_ms,thread_name,conntrack_id,traffic_type",
    );

    // The above writers should write to disk infrequently.

    let mut last_ingress_metrics: Option<IngressMetrics> = None;
    let mut last_print = Instant::now();

    loop {
        if let Some(metrics) = consume_ingress_metrics(&mut ingress_writer, ingress_rx) {
            last_ingress_metrics = Some(metrics);
        }
        consume_classify_metrics(&mut classify_writer, classify_rx);

        if last_print.elapsed() > PRINT_INTERVAL {
            if let Some(m) = &last_ingress_metrics {
                log_ingress_summary(m);
            }
            ingress_writer
                .flush()
                .expect("Failed to flush ingress metrics to disk.");
            classify_writer
                .flush()
                .expect("Failed to flush classify metrics to disk.");
            last_print = Instant::now();
        }

        // Don't eat the CPU.
        thread::sleep(LOOP_SLEEP);
    }
}

/// Try and receive the next set of ingress metrics. Non-blocking.
///
/// # Arguments
///
/// * `writer`: Place to write metric logs.
/// * `ingress_rx`: Channel to receive metrics from.
///
/// # Returns
///
/// The next set of ingress metrics.
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
                    "{},{},{:.6},{},{},{},{},{},{},{},{},{},{}",
                    OffsetDateTime::from(m.timestamp).format(&Rfc3339).unwrap(),
                    m.timestamp.duration_since(UNIX_EPOCH).unwrap().as_millis(),
                    m.interval.as_secs_f64(),
                    m.flow_count,
                    m.classify_backpressure,
                    m.packet_total,
                    m.byte_total,
                    m.packet_interval,
                    m.byte_interval,
                    m.unoptimised_packet_total,
                    m.unoptimised_byte_total,
                    m.unoptimised_packet_interval,
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

/// Try and receive the next set of classify metrics. Non-blocking.
///
/// # Arguments
///
/// * `writer`: Place to write metric logs.
/// * `ingress_rx`: Channel to receive metrics from.
///
/// # Returns
///
/// The next set of classify metrics.
fn consume_classify_metrics(writer: &mut BufWriter<File>, classify_rx: &Receiver<ClassifyMetrics>) {
    loop {
        match classify_rx.try_recv() {
            Ok(m) => {
                writeln!(
                    writer,
                    "{},{},{},{},{}",
                    OffsetDateTime::from(m.timestamp).format(&Rfc3339).unwrap(),
                    m.timestamp.duration_since(UNIX_EPOCH).unwrap().as_millis(),
                    m.thread_name,
                    m.conntrack_id,
                    m.traffic_type,
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

/// Log a summary of the given ingress metrics.
///
/// # Arguments
///
/// * `m`: Metrics to summarize.
#[allow(clippy::cast_precision_loss)]
fn log_ingress_summary(m: &IngressMetrics) {
    let interval_secs = m.interval.as_secs_f64();
    if interval_secs == 0.0 {
        return;
    }

    let packet_rate = m.packet_interval as f64 / interval_secs;
    let mb_rate = (m.byte_interval as f64 / interval_secs) * 8.0 / 1_000_000.0;
    let unoptimised_packet_rate = m.unoptimised_packet_interval as f64 / interval_secs;
    let unoptimised_percent = if m.packet_interval == 0 {
        0.0
    } else {
        m.unoptimised_packet_interval as f64 / m.packet_interval as f64 * 100.0
    };

    info!(
        "Current: {:.2}p/s @ {:.2}Mb/s, Unoptimised {:.2}% @ {:.2}p/s, Jobs {}, Flows {}, Total {} packets @ {:.2}Gb",
        packet_rate,
        mb_rate,
        unoptimised_percent,
        unoptimised_packet_rate,
        m.classify_backpressure,
        m.flow_count,
        m.packet_total,
        (m.byte_total as f64 * 8.0) / 1_000_000_000.0,
    );
}

/// Create new log file to write to.
///
/// # Arguments
///
/// * `path`: The file path to create/open.
/// * `header`: The file header for new files.
///
/// # Returns
///
/// The new file.
fn open_csv_writer(path: &str, header: &str) -> BufWriter<File> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .unwrap_or_else(|_| panic!("Failed to open {path}"));

    let is_empty = file
        .metadata()
        .unwrap_or_else(|_| panic!("Failed to inspect {path}"))
        .len()
        == 0;
    let mut writer = BufWriter::new(file);
    if is_empty {
        writeln!(writer, "{header}").unwrap_or_else(|_| panic!("Failed to write header to {path}"));
    }

    writer
}
