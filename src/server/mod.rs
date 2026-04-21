use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::net::Ipv4Addr;

use crate::classification::model::{Classification, TrafficClassifier, TrafficType};
use nfq::{Queue, Verdict};

const MODEL_PATH: &str = "Packet-Classifier/out/model.onnx";
const DEFAULT_MARK: u32 = 0x801;
const QUEUE_NUM: u16 = 10;

struct ForwardDecision {
    mark: u32,
    classification: Option<Classification>,
    error: Option<String>,
}

pub fn redirect() -> Result<(), Box<dyn std::error::Error>> {
    let mut classifier = open_classifier()?;
    let mut queue = open_queue()?;
    let mut packet_count = 0;

    loop {
        let mut msg = queue.recv()?;
        let decision = select_forward_mark(&mut classifier, &msg);

        log_packet(packet_count, &msg, &decision);
        msg.set_nfmark(decision.mark);
        msg.set_verdict(Verdict::Accept);
        queue.verdict(msg)?;

        packet_count += 1;
    }
}

fn open_queue() -> std::io::Result<Queue> {
    println!("Opening netfilter queue {QUEUE_NUM}...");
    let mut queue = Queue::open()?;
    queue.bind(QUEUE_NUM)?;
    queue.set_recv_conntrack(QUEUE_NUM, true)?;
    println!("Netfilter queue {QUEUE_NUM} ready.");
    Ok(queue)
}

fn open_classifier() -> Result<TrafficClassifier, Box<dyn std::error::Error>> {
    println!("Loading traffic classifier from {MODEL_PATH}...");
    let classifier = TrafficClassifier::from_file(MODEL_PATH)?;
    println!("Traffic classifier ready.");
    Ok(classifier)
}

fn select_forward_mark(classifier: &mut TrafficClassifier, msg: &nfq::Message) -> ForwardDecision {
    let payload = msg.get_payload();
    let classifier_payload = transport_payload(payload).unwrap_or(payload);

    match classifier.classify_payload(classifier_payload) {
        Ok(classification) => ForwardDecision {
            mark: mark_for_traffic_type(classification.traffic_type),
            classification: Some(classification),
            error: None,
        },
        Err(error) => ForwardDecision {
            mark: DEFAULT_MARK,
            classification: None,
            error: Some(error.to_string()),
        },
    }
}

fn mark_for_traffic_type(traffic_type: TrafficType) -> u32 {
    match traffic_type {
        TrafficType::GoogleMeet => 0x801,
        TrafficType::Instagram => 0x802,
        TrafficType::TikTok => 0x802,
        TrafficType::Twitter => 0x802,
        TrafficType::Youtube => 0x801,
    }
}

fn log_packet(packet_count: u64, msg: &nfq::Message, decision: &ForwardDecision) {
    let payload = msg.get_payload();

    println!(
        "rx {packet_count}: packet_id={}, queue={}",
        msg.get_packet_id(),
        msg.get_queue_num()
    );
    println!(
        "  size: payload={} original={} hash={:#X}",
        payload.len(),
        msg.get_original_len(),
        payload_hash(payload)
    );
    println!(
        "  offload: gso={} checksum_ready={}",
        msg.is_seg_offloaded(),
        msg.is_checksum_ready()
    );
    print_process_metadata(msg);
    match describe_ipv4_packet(payload) {
        Some(desc) => println!("  flow: {desc}"),
        None => println!("  flow: unable to parse IPv4 header"),
    }
    print_classification(decision);
    print_conntrack(msg.get_conntrack());
    println!("  mark: 0x{:X} -> 0x{:X}", msg.get_nfmark(), decision.mark);
}

fn print_classification(decision: &ForwardDecision) {
    if let Some(classification) = &decision.classification {
        println!(
            "  classification: {} confidence={:.4}",
            classification.traffic_type,
            top_score(&classification.scores)
        );
    } else if let Some(error) = &decision.error {
        println!("  classification: unavailable ({error})");
    }
}

fn top_score(scores: &[f32]) -> f32 {
    scores.iter().copied().fold(f32::NEG_INFINITY, f32::max)
}

fn payload_hash(payload: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    payload.hash(&mut hasher);
    hasher.finish()
}

fn print_process_metadata(msg: &nfq::Message) {
    let uid = msg
        .get_uid()
        .map_or_else(|| "-".to_string(), |uid| uid.to_string());
    let gid = msg
        .get_gid()
        .map_or_else(|| "-".to_string(), |gid| gid.to_string());
    let timestamp = msg
        .get_timestamp()
        .map_or_else(|| "-".to_string(), |timestamp| format!("{timestamp:?}"));

    println!("  process: uid={uid} gid={gid} timestamp={timestamp}");
}

fn print_conntrack(conntrack: Option<&nfq::Conntrack>) {
    match conntrack {
        Some(conntrack) => println!(
            "  conntrack: id={} state={:?}",
            conntrack.get_id(),
            conntrack.get_state()
        ),
        None => println!("  conntrack: unavailable"),
    }
}

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
