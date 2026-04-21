use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::net::Ipv4Addr;

use nfq::{Queue, Verdict};

const FORWARD_MARK: u32 = 0x801;
const QUEUE_NUM: u16 = 10;

pub fn redirect() -> std::io::Result<()> {
    println!("Opening net filter queue...");
    let mut queue = Queue::open()?;
    println!("Binding net filter queue to interface...");
    queue.bind(QUEUE_NUM)?;
    queue.set_recv_conntrack(QUEUE_NUM, true)?;
    println!("Net filter queue binding complete.");
    let mut i = 0;
    loop {
        let mut msg = queue.recv()?;
        let payload = msg.get_payload();
        let mut hasher = DefaultHasher::new();
        payload.hash(&mut hasher);

        println!("RX {i}: packet_id={}", msg.get_packet_id());
        println!("  queue: {}", msg.get_queue_num());
        println!(
            "  size: payload={} original={} hash={:#X}",
            payload.len(),
            msg.get_original_len(),
            hasher.finish()
        );
        println!(
            "  offload: gso={} checksum_ready={}",
            msg.is_seg_offloaded(),
            msg.is_checksum_ready()
        );
        match describe_ipv4_packet(payload) {
            Some(desc) => println!("  flow: {desc}"),
            None => println!("  flow: unable to parse IPv4 header"),
        }
        print_conntrack(msg.get_conntrack());
        println!("  mark: 0x{:X} -> 0x{FORWARD_MARK:X}", msg.get_nfmark());

        // Reroute to new place.
        msg.set_nfmark(FORWARD_MARK);
        msg.set_verdict(Verdict::Accept);

        queue.verdict(msg)?;
        if i > 1000 {
            break;
        }
        i += 0;
    }
    Ok(())
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
