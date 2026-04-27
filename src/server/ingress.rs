use std::cmp::PartialEq;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::net::Ipv4Addr;
use std::time::{Duration, Instant};

use crate::classification::model::TrafficType;
pub(crate) use crate::server::{ClassifyResult, ClassifyTask, ConntrackId};
use crossbeam_channel::{Receiver, Sender, TryRecvError, TrySendError};
use nfq::{Queue, Verdict};

type FwMark = u32;
const DEFAULT_MARK: FwMark = 0x801;
const PACKETS_FOR_CLASSIFY: usize = 5;
const PRUNE_FLOWS_OLDER_THAN: Duration = Duration::from_secs(10);

/// Represents the classification status
/// regarding a specific conntrack flow.
#[derive(Clone, Debug, Eq, PartialEq)]
enum ClassifyStatus {
    Collecting,
    Classifying,
    Pinned(FwMark),
}

/// Represents the state of a given network flow,
/// associated with a given ConntrackId.
struct FlowState {
    buf: Vec<Vec<u8>>,
    //collected_bytes: u32,
    status: ClassifyStatus,
    last_seen: Instant,
}

/// Act as the ingress for packets into the system. This function reads in packets
/// and decides when to start classification tasks and marks packets accordingly.
///
/// # Arguments
///
/// * `nfqueue`: netfilter queue to receive packets.
/// * `task_tx`: channel to send packet classification tasks
/// * `result_rx`: channel to receive packet classification results
///
/// returns: ()
pub fn ingress_loop(
    nfqueue: &mut Queue,
    task_tx: Sender<ClassifyTask>,
    result_rx: Receiver<ClassifyResult>,
) {
    let mut flow_map: HashMap<ConntrackId, FlowState> = HashMap::new();
    let mut packet_count = 0usize;
    let mut byte_count = 0usize;

    loop {
        // Rx classify results.
        consume_results(&result_rx, &mut flow_map);

        // Rx packet.
        let mut msg = nfqueue.recv().unwrap();
        let payload = msg.get_payload().to_vec();
        let conntrack = *(&msg
            .get_conntrack()
            .expect("Failed to retrieve conntrack information.")
            .get_id()) as ConntrackId;

        // Tx classify task.
        let status = handle_classification(&task_tx, &mut flow_map, conntrack, payload);

        // Log.
        packet_count += 1;
        byte_count += msg.get_payload().len();
        //log_packet(&msg, payload.clone(), conntrack, packet_count, byte_count);
        //println!("RX={}, Conntrack={}", packet_count, conntrack);

        // Tx packet.
        msg.set_nfmark(match status {
            ClassifyStatus::Collecting => DEFAULT_MARK,
            ClassifyStatus::Classifying => DEFAULT_MARK,
            ClassifyStatus::Pinned(x) => x,
        });
        msg.set_verdict(Verdict::Accept);
        nfqueue.verdict(msg).unwrap();

        // Prune stale flows every 10 packets.
        if packet_count % 10 == 1 {
            prune_stale_flows(&mut flow_map);
        }
    }
}

///
///
/// # Arguments
///
/// * `result_rx`:
/// * `flow_map`:
///
/// returns: ()
fn consume_results(
    result_rx: &Receiver<ClassifyResult>,
    flow_map: &mut HashMap<ConntrackId, FlowState>,
) {
    loop {
        match result_rx.try_recv() {
            Ok(result) => {
                if let Some(flow) = flow_map.get_mut(&result.id) {
                    flow.status = match result.classification {
                        Ok(v) => ClassifyStatus::Pinned(mark_for_traffic_type(v)),
                        Err(_) => ClassifyStatus::Pinned(DEFAULT_MARK),
                    };
                    flow.last_seen = Instant::now();
                }
            }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => break,
        }
    }
}

///
///
/// # Arguments
///
/// * `task_tx`:
/// * `flow_map`:
/// * `conntrack`:
/// * `payload`:
///
/// returns: ClassifyStatus
fn handle_classification(
    task_tx: &Sender<ClassifyTask>,
    flow_map: &mut HashMap<ConntrackId, FlowState>,
    conntrack: ConntrackId,
    payload: Vec<u8>,
) -> ClassifyStatus {
    if let Some(flow_state) = flow_map.get_mut(&conntrack) {
        flow_state.buf.push(payload);
        flow_state.last_seen = Instant::now();

        // Tx classify task if ready.
        if flow_state.status == ClassifyStatus::Collecting
            && flow_state.buf.len() >= PACKETS_FOR_CLASSIFY - 1
        {
            match task_tx.try_send(ClassifyTask {
                id: conntrack,
                buf: flow_state.buf.clone(),
            }) {
                Ok(_) => {}
                Err(TrySendError::Disconnected(_)) => {
                    eprintln!("Failed to send classify job. Channel disconnected.")
                }
                Err(TrySendError::Full(_)) => {
                    eprintln!("Failed to send classify job. Channel full.")
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
        println!("New conntrack={}", conntrack);
        ClassifyStatus::Collecting
    }
}

///
///
/// # Arguments
///
/// * `traffic_type`:
///
/// returns: u32
fn mark_for_traffic_type(traffic_type: TrafficType) -> u32 {
    match traffic_type {
        TrafficType::GoogleMeet => 0x801,
        TrafficType::Instagram => 0x802,
        TrafficType::TikTok => 0x803,
        TrafficType::Twitter => 0x804,
        TrafficType::Youtube => 0x801,
    }
}

/// Remove old flows.
///
/// # Arguments
///
/// * `flow_map`:
fn prune_stale_flows(flow_map: &mut HashMap<ConntrackId, FlowState>) {
    // Todo
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

fn describe_ipv4_packet(payload: &[u8]) -> Option<String> {
    if payload.len() < 20 {
        return None;
    }

    let version = payload[0] >> 4;
    if version != 4 {
        return Some(format!("packet: non-IPv4 version {version}"));
    }

    let header_len = usize::from(payload[0] & 0x0F) * 4;
    if header_len < 20 || payload.len() < header_len {
        return None;
    }

    let total_len = u16::from_be_bytes([payload[2], payload[3]]);
    let protocol = payload[9];
    let src = Ipv4Addr::new(payload[12], payload[13], payload[14], payload[15]);
    let dst = Ipv4Addr::new(payload[16], payload[17], payload[18], payload[19]);

    match protocol {
        1 => describe_icmp(payload, header_len, total_len, src, dst),
        6 => describe_ports("tcp", payload, header_len, total_len, src, dst),
        17 => describe_ports("udp", payload, header_len, total_len, src, dst),
        _ => Some(format!(
            "ipv4 {src} -> {dst} proto={protocol} total_len={total_len}"
        )),
    }
}

fn transport_payload(payload: &[u8]) -> Option<&[u8]> {
    let header_len = ipv4_header_len(payload)?;
    match payload[9] {
        6 => tcp_payload(payload, header_len),
        17 => udp_payload(payload, header_len),
        _ => None,
    }
}

fn ipv4_header_len(payload: &[u8]) -> Option<usize> {
    if payload.len() < 20 || payload[0] >> 4 != 4 {
        return None;
    }

    let header_len = usize::from(payload[0] & 0x0F) * 4;
    if header_len < 20 || payload.len() < header_len {
        return None;
    }

    Some(header_len)
}

fn udp_payload(payload: &[u8], ip_header_len: usize) -> Option<&[u8]> {
    let udp_header_len = 8;
    let payload_offset = ip_header_len + udp_header_len;
    if payload.len() < payload_offset {
        return None;
    }

    Some(&payload[payload_offset..])
}

fn tcp_payload(payload: &[u8], ip_header_len: usize) -> Option<&[u8]> {
    if payload.len() < ip_header_len + 13 {
        return None;
    }

    let tcp_header_len = usize::from(payload[ip_header_len + 12] >> 4) * 4;
    let payload_offset = ip_header_len + tcp_header_len;
    if tcp_header_len < 20 || payload.len() < payload_offset {
        return None;
    }

    Some(&payload[payload_offset..])
}

fn describe_ports(
    protocol_name: &str,
    payload: &[u8],
    header_len: usize,
    total_len: u16,
    src: Ipv4Addr,
    dst: Ipv4Addr,
) -> Option<String> {
    if payload.len() < header_len + 4 {
        return None;
    }

    let src_port = u16::from_be_bytes([payload[header_len], payload[header_len + 1]]);
    let dst_port = u16::from_be_bytes([payload[header_len + 2], payload[header_len + 3]]);

    Some(format!(
        "{protocol_name} {src}:{src_port} -> {dst}:{dst_port} total_len={total_len}"
    ))
}

fn describe_icmp(
    payload: &[u8],
    header_len: usize,
    total_len: u16,
    src: Ipv4Addr,
    dst: Ipv4Addr,
) -> Option<String> {
    if payload.len() < header_len + 2 {
        return None;
    }

    Some(format!(
        "icmp {src} -> {dst} type={} code={} total_len={total_len}",
        payload[header_len],
        payload[header_len + 1]
    ))
}
