use std::cmp::PartialEq;
use std::collections::HashMap;
//use std::collections::hash_map::DefaultHasher;
//use std::net::Ipv4Addr;
use std::time::{Duration, Instant};

use crate::classification::model::TrafficType;
use crate::server::classify::{ClassifyResult, ClassifyTask};
use crossbeam_channel::{Receiver, Sender, TryRecvError, TrySendError};
use nfq::{Queue, Verdict};
use tracing::{debug, error, info, warn};

/// Represents a kernel-level conntrack ID for a given flow.
pub type ConntrackId = u32;

/// Represents a packet firewall mark.
type FwMark = u32;

const DEFAULT_MARK: FwMark = 0x801;
const PACKETS_FOR_CLASSIFY: usize = 5;
const STALE_FLOW_PRUNE: Duration = Duration::from_secs(30);
const LOG_INTERVAL: Duration = Duration::from_secs(1);

/// Classification progress for a tracked conntrack flow.
///
/// A flow starts in [`ClassifyStatus::Collecting`], moves to
/// [`ClassifyStatus::Classifying`] once enough packets have been buffered for a
/// job, and is finally pinned to a forwarding mark after inference completes.
#[derive(Clone, Debug, Eq, PartialEq)]
enum ClassifyStatus {
    Collecting,
    Classifying,
    Pinned(FwMark),
}

/// Mutable state held for each observed conntrack flow.
struct FlowState {
    buf: Vec<Vec<u8>>,
    //collected_bytes: u32,
    status: ClassifyStatus,
    last_seen: Instant,
}

/// Act as the ingress loop for packets entering the forwarding pipeline.
///
/// # Arguments
///
/// * `nfqueue`: Netfilter queue to receive packets.
/// * `task_tx`: Channel used to enqueue classification work for worker threads.
/// * `result_rx`: Channel used to receive completed classification results.
#[allow(clippy::cast_precision_loss)]
pub fn ingress_loop(
    nfqueue: &mut Queue,
    task_tx: &Sender<ClassifyTask>,
    result_rx: &Receiver<ClassifyResult>,
) {
    let mut flow_map: HashMap<ConntrackId, FlowState> = HashMap::new();
    let mut packet_count = 0usize;
    let mut byte_count = 0usize;
    let mut packet_interval = 0usize;
    let mut byte_interval = 0usize;
    let mut last_log_interval = Instant::now();
    let mut last_pruned = Instant::now();
    let mut inefficient_count = 0usize;
    let mut inefficient_interval = 0usize;

    loop {
        // Rx classify results.
        consume_results(result_rx, &mut flow_map);

        // Rx packet.
        let mut msg = nfqueue.recv().unwrap();
        let payload = msg.get_payload().to_vec();
        let payload_len = payload.len();
        let conntrack = msg
            .get_conntrack()
            .expect("Failed to retrieve conntrack information.")
            .get_id() as ConntrackId;

        // Tx classify task.
        let status = handle_classification(task_tx, &mut flow_map, conntrack, payload);

        // Tx packet.
        match status {
            ClassifyStatus::Collecting | ClassifyStatus::Classifying => {}
            ClassifyStatus::Pinned(mark) => msg.set_nfmark(mark),
        }
        msg.set_verdict(Verdict::Accept);
        nfqueue.verdict(msg).unwrap();

        // Metrics.
        packet_count += 1;
        packet_interval += 1;
        byte_count += payload_len;
        byte_interval += payload_len;
        if status == ClassifyStatus::Classifying {
            inefficient_count += 1;
            inefficient_interval += 1;
        }

        // Prune old connections.
        if last_pruned.elapsed() > STALE_FLOW_PRUNE {
            let old_conns = flow_map.len();
            flow_map.retain(|_, flow_state| flow_state.last_seen.elapsed() < STALE_FLOW_PRUNE);
            let new_conns = flow_map.len();
            if old_conns != new_conns {
                info!("Pruned connections: {:} -> {}", old_conns, new_conns);
            }
            last_pruned = Instant::now();
        }

        // Log throughput.
        if last_log_interval.elapsed() > LOG_INTERVAL {
            let time_delay = last_log_interval.elapsed().as_secs_f64();
            info!(
                "Total: {} packets @ {:.2} Gb, Missed: {}",
                packet_count,
                (byte_count as f64 * 8.0) / 1_000_000_000.0,
                inefficient_count
            );
            info!(
                "Current: {:.2}p/s @ {:.2}Mb/s, Missed {:.2}% @ {:.2}p/s",
                packet_interval as f64 / time_delay,
                (byte_interval as f64 / time_delay) * 8.0 / 1_000_000.0,
                inefficient_interval as f64 / packet_interval as f64 * 100.0,
                inefficient_interval as f64 / time_delay,
            );

            packet_interval = 0;
            byte_interval = 0;
            inefficient_interval = 0;
            last_log_interval = Instant::now();
        }
    }
}
/// Drain all currently available classification results from the worker queue.
///
/// # Arguments
///
/// * `result_rx`: Receiver for completed classification results.
/// * `flow_map`: Per-flow state table keyed by conntrack ID.
fn consume_results(
    result_rx: &Receiver<ClassifyResult>,
    flow_map: &mut HashMap<ConntrackId, FlowState>,
) {
    loop {
        match result_rx.try_recv() {
            Ok(result) => {
                if let Some(flow) = flow_map.get_mut(&result.id) {
                    flow.status = match result.classification {
                        Ok(v) => {
                            info!(
                                "Directing conntrack {:#010X} -> {:#X?}",
                                &result.id,
                                mark_for_traffic_type(v)
                            );
                            ClassifyStatus::Pinned(mark_for_traffic_type(v))
                        }
                        Err(_) => ClassifyStatus::Pinned(DEFAULT_MARK),
                    };
                    flow.last_seen = Instant::now();
                }
            }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => {
                error!("Failed to receive classify result. Channel disconnected.");
                break;
            }
        }
    }
}

/// Update flow state for an incoming packet and enqueue classification if ready.
///
/// Existing flows continue buffering payloads until the configured packet count
/// is reached. New flows are inserted into the tracking table and begin in the
/// collecting state.
///
/// # Arguments
///
/// * `task_tx`: Sender used to dispatch a completed classification batch.
/// * `flow_map`: Mutable per-flow state table.
/// * `conntrack`: Kernel conntrack identifier for the packet's flow.
/// * `payload`: Raw packet payload to append to the flow buffer.
///
/// # Returns
///
/// The flow's current [`ClassifyStatus`] after processing this packet.
fn handle_classification(
    task_tx: &Sender<ClassifyTask>,
    flow_map: &mut HashMap<ConntrackId, FlowState>,
    conntrack: ConntrackId,
    payload: Vec<u8>,
) -> ClassifyStatus {
    if let Some(flow_state) = flow_map.get_mut(&conntrack) {
        flow_state.last_seen = Instant::now();
        if flow_state.status != ClassifyStatus::Collecting {
            return flow_state.status.clone();
        }
        flow_state.buf.push(payload);

        // Tx classify task if ready.
        if flow_state.buf.len() >= PACKETS_FOR_CLASSIFY {
            match task_tx.try_send(ClassifyTask {
                id: conntrack,
                buf: flow_state.buf.clone(),
            }) {
                Ok(()) => {}
                Err(TrySendError::Disconnected(_)) => {
                    error!("Failed to send classify job. Channel disconnected.");
                }
                Err(TrySendError::Full(_)) => {
                    warn!("Failed to send classify job. Channel full.");
                }
            }
            flow_state.status = ClassifyStatus::Classifying;
        }
        flow_state.status.clone()
    } else {
        // Track new connection.
        let mut new_buf: Vec<Vec<u8>> = Vec::with_capacity(PACKETS_FOR_CLASSIFY);
        new_buf.push(payload);
        flow_map.insert(
            conntrack,
            FlowState {
                buf: new_buf,
                status: ClassifyStatus::Collecting,
                last_seen: Instant::now(),
            },
        );
        debug!("New conntrack={:#010X?}", conntrack);
        ClassifyStatus::Collecting
    }
}

/// Map a predicted traffic class to the firewall mark used by forwarding.
///
/// # Arguments
///
/// * `traffic_type`: Predicted application traffic class.
///
/// # Returns
///
/// The `nfmark` value that should be applied to packets from that flow.
fn mark_for_traffic_type(traffic_type: TrafficType) -> u32 {
    match traffic_type {
        TrafficType::GoogleMeet => 0x801,
        TrafficType::Instagram => 0x802,
        TrafficType::TikTok => 0x803,
        TrafficType::Twitter => 0x804,
        TrafficType::Youtube => 0x802,
    }
}

// fn log_packet(packet_count: usize, msg: &nfq::Message, decision: &ForwardDecision) {
//     let payload = msg.get_payload();
//
//     println!(
//         "rx {packet_count}: packet_id={}, queue={}",
//         msg.get_packet_id(),
//         msg.get_queue_num()
//     );
//     // println!(
//     //     "  size: payload={} original={} hash={:#X}",
//     //     payload.len(),
//     //     msg.get_original_len(),
//     //     payload_hash(payload)
//     // );
//     // println!(
//     //     "  offload: gso={} checksum_ready={}",
//     //     msg.is_seg_offloaded(),
//     //     msg.is_checksum_ready()
//     // );
//     // match describe_ipv4_packet(payload) {
//     //     Some(desc) => println!("  flow: {desc}"),
//     //     None => println!("  flow: unable to parse IPv4 header"),
//     // }
//     // print_classification(decision);
//     print_conntrack(msg.get_conntrack());
//     println!(
//         "  fwmark: 0x{:X} -> 0x{:X}",
//         msg.get_nfmark(),
//         decision.mark
//     );
// }
//
// fn print_classification(decision: &ForwardDecision) {
//     if let Some(classification) = &decision.classification {
//         println!(
//             "  classification: {}, confidence={:.4}",
//             classification.traffic_type,
//             top_score(&classification.scores)
//         );
//     } else if let Some(error) = &decision.error {
//         println!("  classification: unavailable ({error})");
//     }
// }
//
// fn top_score(scores: &[f32]) -> f32 {
//     scores.iter().copied().fold(f32::NEG_INFINITY, f32::max)
// }
//
// fn payload_hash(payload: &[u8]) -> u64 {
//     let mut hasher = DefaultHasher::new();
//     payload.hash(&mut hasher);
//     hasher.finish()
// }
//
// fn describe_ipv4_packet(payload: &[u8]) -> Option<String> {
//     if payload.len() < 20 {
//         return None;
//     }
//
//     let version = payload[0] >> 4;
//     if version != 4 {
//         return Some(format!("packet: non-IPv4 version {version}"));
//     }
//
//     let header_len = usize::from(payload[0] & 0x0F) * 4;
//     if header_len < 20 || payload.len() < header_len {
//         return None;
//     }
//
//     let total_len = u16::from_be_bytes([payload[2], payload[3]]);
//     let protocol = payload[9];
//     let src = Ipv4Addr::new(payload[12], payload[13], payload[14], payload[15]);
//     let dst = Ipv4Addr::new(payload[16], payload[17], payload[18], payload[19]);
//
//     match protocol {
//         1 => describe_icmp(payload, header_len, total_len, src, dst),
//         6 => describe_ports("tcp", payload, header_len, total_len, src, dst),
//         17 => describe_ports("udp", payload, header_len, total_len, src, dst),
//         _ => Some(format!(
//             "ipv4 {src} -> {dst} proto={protocol} total_len={total_len}"
//         )),
//     }
// }
//
// fn transport_payload(payload: &[u8]) -> Option<&[u8]> {
//     let header_len = ipv4_header_len(payload)?;
//     match payload[9] {
//         6 => tcp_payload(payload, header_len),
//         17 => udp_payload(payload, header_len),
//         _ => None,
//     }
// }
//
// fn ipv4_header_len(payload: &[u8]) -> Option<usize> {
//     if payload.len() < 20 || payload[0] >> 4 != 4 {
//         return None;
//     }
//
//     let header_len = usize::from(payload[0] & 0x0F) * 4;
//     if header_len < 20 || payload.len() < header_len {
//         return None;
//     }
//
//     Some(header_len)
// }
//
// fn udp_payload(payload: &[u8], ip_header_len: usize) -> Option<&[u8]> {
//     let udp_header_len = 8;
//     let payload_offset = ip_header_len + udp_header_len;
//     if payload.len() < payload_offset {
//         return None;
//     }
//
//     Some(&payload[payload_offset..])
// }
//
// fn tcp_payload(payload: &[u8], ip_header_len: usize) -> Option<&[u8]> {
//     if payload.len() < ip_header_len + 13 {
//         return None;
//     }
//
//     let tcp_header_len = usize::from(payload[ip_header_len + 12] >> 4) * 4;
//     let payload_offset = ip_header_len + tcp_header_len;
//     if tcp_header_len < 20 || payload.len() < payload_offset {
//         return None;
//     }
//
//     Some(&payload[payload_offset..])
// }
//
// fn describe_ports(
//     protocol_name: &str,
//     payload: &[u8],
//     header_len: usize,
//     total_len: u16,
//     src: Ipv4Addr,
//     dst: Ipv4Addr,
// ) -> Option<String> {
//     if payload.len() < header_len + 4 {
//         return None;
//     }
//
//     let src_port = u16::from_be_bytes([payload[header_len], payload[header_len + 1]]);
//     let dst_port = u16::from_be_bytes([payload[header_len + 2], payload[header_len + 3]]);
//
//     Some(format!(
//         "{protocol_name} {src}:{src_port} -> {dst}:{dst_port} total_len={total_len}"
//     ))
// }
//
// fn describe_icmp(
//     payload: &[u8],
//     header_len: usize,
//     total_len: u16,
//     src: Ipv4Addr,
//     dst: Ipv4Addr,
// ) -> Option<String> {
//     if payload.len() < header_len + 2 {
//         return None;
//     }
//
//     Some(format!(
//         "icmp {src} -> {dst} type={} code={} total_len={total_len}",
//         payload[header_len],
//         payload[header_len + 1]
//     ))
// }
