use std::cmp::PartialEq;
use std::collections::HashMap;
use std::collections::hash_map::Entry::Vacant;
use std::env;
use std::time::{Duration, Instant, SystemTime};

use crate::classification::model::TrafficType;
pub(crate) use crate::server::classify::{ClassifyResult, ClassifyTask};
use crate::server::monitor::IngressMetrics;
use crossbeam_channel::{Receiver, Sender, TryRecvError, TrySendError};
use nfq::{Queue, Verdict};
use tracing::{debug, error, info, warn};

/// Represents a kernel-level conntrack ID for a given flow.
pub type ConntrackId = u32;

/// Represents a packet firewall mark.
type FwMark = u32;

/// Represents
const DEFAULT_MARK: FwMark = 0x801;
const FLOW_CLASSIFY_BYTES: usize = 28 * 28;
const MIN_PACKETS_FOR_CLASSIFY: usize = 5;
/// Represents How often stale flows should be removed/unclassified.
const FLOW_PRUNE_INTERVAL: Duration = Duration::from_secs(30);
/// Represents how often metrics should be pushed.
const LOG_INTERVAL: Duration = Duration::from_secs(1);

/// Represents how the system should handle packets.
#[derive(Clone, Copy, Debug)]
enum RouteMode {
    /// Accept all packets.
    None,
    /// Block traffic for classification, then direct.
    Blocking,
    /// Accept traffic before classification, then redirect.
    /// Changes in NAT mid-flow can break connection based protocols.
    NonBlocking,
}

impl RouteMode {
    fn from_env() -> Self {
        match env::var("PFS_ROUTE_MODE") {
            Ok(value) if value.eq_ignore_ascii_case("none") => Self::None,
            Ok(value) if value.eq_ignore_ascii_case("blocking") => Self::Blocking,
            Ok(value) if value.eq_ignore_ascii_case("nonblocking") => Self::NonBlocking,
            Ok(value) if value.eq_ignore_ascii_case("non-blocking") => Self::NonBlocking,
            Ok(_) | Err(_) => Self::NonBlocking,
        }
    }
}

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
    /// The accumulated bytes of an observed flow.
    sample: Vec<u8>,
    packets_seen: usize,
    status: ClassifyStatus,
    last_seen: Instant,
}

///
enum ForwardingAction {
    Accept { mark: Option<FwMark> },
    // Block,
}

/// Act as the ingress loop for packets entering the forwarding pipeline.
///
/// # Arguments
///
/// * `nfqueue`: Netfilter queue to receive packets.
/// * `task_tx`: Channel used to enqueue classification work for worker threads.
/// * `result_rx`: Channel used to receive completed classification results.
/// * `metrics_tx`: Channel
pub fn ingress_loop(
    nfqueue: &mut Queue,
    task_tx: &Sender<ClassifyTask>,
    result_rx: &Receiver<ClassifyResult>,
    metrics_tx: &Sender<IngressMetrics>,
) {
    let route_mode = RouteMode::from_env();
    let mut flow_map: HashMap<ConntrackId, FlowState> = HashMap::new();
    let mut packet_total = 0usize;
    let mut byte_total = 0usize;
    let mut packet_interval = 0usize;
    let mut byte_interval = 0usize;
    let mut unoptimised_packet_total = 0usize;
    let mut unoptimised_packet_interval = 0usize;
    let mut unoptimised_byte_total = 0usize;
    let mut unoptimised_byte_interval = 0usize;

    let mut last_metric_push = Instant::now();
    let mut last_pruned = Instant::now();

    info!("Route mode = {route_mode:?} (set with PFS_ROUTE_MODE)");
    info!("Metrics polling interval = {LOG_INTERVAL:?}");

    loop {
        // Rx classify results.
        consume_results(result_rx, &mut flow_map);

        // Rx packet.
        let mut msg = nfqueue
            .recv()
            .expect("Failed to receive packet from Netfilter queue.");
        let packet = msg.get_payload();
        let payload_len = packet.len();
        let conntrack = msg
            .get_conntrack()
            .expect("Failed to retrieve conntrack information.")
            .get_id() as ConntrackId;

        // Tx classify task.
        let status = handle_classification(task_tx, &mut flow_map, conntrack, packet);

        // Tx packet.
        match forwarding_action(&status, route_mode) {
            ForwardingAction::Accept { mark } => {
                if let Some(mark) = mark {
                    msg.set_nfmark(mark);
                }
                else {
                    msg.set_nfmark(0x801);
                }
                msg.set_verdict(Verdict::Accept);
                nfqueue.verdict(msg).expect("Failed to forward message.");
            } // ForwardingAction::Block => {
              //     panic!("Not implemented!");
              // }
        }

        // Prune old connections.
        if last_pruned.elapsed() > FLOW_PRUNE_INTERVAL {
            flow_map.retain(|_, flow_state| flow_state.last_seen.elapsed() < FLOW_PRUNE_INTERVAL);
            last_pruned = Instant::now();
        }

        // Accumulate metrics.
        packet_total += 1;
        packet_interval += 1;
        byte_total += payload_len;
        byte_interval += payload_len;
        if status == ClassifyStatus::Classifying {
            // Packets that make it here are being forwarded before they can be classified.
            unoptimised_packet_total += 1;
            unoptimised_packet_interval += 1;
            unoptimised_byte_total += payload_len;
            unoptimised_byte_interval += payload_len;
        }

        // Push metrics.
        if last_metric_push.elapsed() > LOG_INTERVAL {
            let metrics_interval = last_metric_push.elapsed();
            match metrics_tx.try_send(IngressMetrics {
                timestamp: SystemTime::now(),
                interval: metrics_interval,
                flow_count: flow_map.len(),
                classify_backpressure: task_tx.len(),
                packet_total,
                byte_total,
                packet_interval,
                byte_interval,
                unoptimised_packet_total,
                unoptimised_byte_total,
                unoptimised_packet_interval,
                unoptimised_byte_interval,
            }) {
                Ok(()) => {}
                Err(TrySendError::Disconnected(_)) => {
                    panic!("Failed to send ingress metrics. Channel disconnected.");
                }
                Err(TrySendError::Full(_)) => {
                    // Unexpected, drop metrics.
                    error!("Failed to send ingress metrics. Channel full.");
                }
            }

            // Reset metrics interval.
            packet_interval = 0;
            byte_interval = 0;
            unoptimised_packet_interval = 0;
            unoptimised_byte_interval = 0;
            last_metric_push = Instant::now();
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
                            let mark = mark_for_traffic_type(v);
                            info!(
                                "Directing conntrack {:#010X} -> {} ({:#X?})",
                                &result.id, v, mark
                            );
                            ClassifyStatus::Pinned(mark)
                        }
                        Err(_) => ClassifyStatus::Pinned(DEFAULT_MARK),
                    };
                    flow.last_seen = Instant::now();
                }
            }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => {
                panic!("Failed to receive classify result. Channel disconnected.")
            }
        }
    }
}

/// Update flow state for an incoming packet and enqueue classification if ready.
///
/// Existing flows continue buffering sample bytes until enough flow data is
/// available. New flows are inserted into the tracking table and begin in the
/// collecting state.
///
/// # Arguments
///
/// * `task_tx`: Sender used to dispatch a completed classification batch.
/// * `flow_map`: Mutable per-flow state table.
/// * `conntrack`: Kernel conntrack identifier for the packet's flow.
/// * `sample_bytes`: Packet bytes selected for model input.
///
/// # Returns
///
/// The flow's current [`ClassifyStatus`] after processing this packet.
fn handle_classification(
    task_tx: &Sender<ClassifyTask>,
    flow_map: &mut HashMap<ConntrackId, FlowState>,
    conntrack: ConntrackId,
    sample_bytes: &[u8],
) -> ClassifyStatus {
    if let Vacant(e) = flow_map.entry(conntrack) {
        e.insert(FlowState {
            sample: Vec::with_capacity(FLOW_CLASSIFY_BYTES),
            packets_seen: 0,
            status: ClassifyStatus::Collecting,
            last_seen: Instant::now(),
        });
        debug!("New conntrack={:#010X}", conntrack);
    }

    let flow_state = flow_map
        .get_mut(&conntrack)
        .expect("Flow should exist after insert.");

    flow_state.last_seen = Instant::now();
    if flow_state.status != ClassifyStatus::Collecting {
        return flow_state.status.clone();
    }

    // Accumulate bytes.
    flow_state.packets_seen += 1;
    let available = FLOW_CLASSIFY_BYTES.saturating_sub(flow_state.sample.len());
    let copied_len = sample_bytes.len().min(available);
    flow_state
        .sample
        .extend_from_slice(&sample_bytes[..copied_len]);

    // Tx classify task if ready.
    if flow_state.sample.len() >= FLOW_CLASSIFY_BYTES
        || (flow_state.packets_seen >= MIN_PACKETS_FOR_CLASSIFY && !flow_state.sample.is_empty())
    {
        match task_tx.try_send(ClassifyTask {
            id: conntrack,
            sample: flow_state.sample.clone(),
        }) {
            Ok(()) => {}
            Err(TrySendError::Disconnected(_)) => {
                panic!("Failed to send classify job. Channel disconnected.");
            }
            Err(TrySendError::Full(_)) => {
                // Not a completely unexpected occurrence.
                warn!("Failed to send classify job. Channel full.");
            }
        }
        flow_state.status = ClassifyStatus::Classifying;
    }

    flow_state.status.clone()
}

/// Determine
///
/// # Arguments
///
/// * `status`:
/// * `route_mode`:
///
/// # Returns
///
/// The
fn forwarding_action(status: &ClassifyStatus, route_mode: RouteMode) -> ForwardingAction {
    match route_mode {
        RouteMode::None => ForwardingAction::Accept { mark: None },
        RouteMode::Blocking => {
            // Accumulate packets to be released upon classification.
            panic!("Not implemented.");
        }
        RouteMode::NonBlocking => match status {
            ClassifyStatus::Collecting | ClassifyStatus::Classifying => {
                ForwardingAction::Accept { mark: None }
            }
            ClassifyStatus::Pinned(mark) => ForwardingAction::Accept { mark: Some(*mark) },
        },
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
fn mark_for_traffic_type(traffic_type: TrafficType) -> FwMark {
    match traffic_type {
        TrafficType::GoogleMeet | TrafficType::Youtube => 0x801,
        TrafficType::Instagram | TrafficType::TikTok => 0x802,
        TrafficType::Twitter => 0x803,
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
