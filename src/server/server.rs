use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use nfq::{Queue, Verdict};

pub fn redirect() -> std::io::Result<()> {
    println!("Opening net filter queue...");
    let mut queue = Queue::open()?;
    println!("Binding net filter queue to interface...");
    //let mut hashes = HashMap::new();
    queue.bind(10)?;
    println!("Net filter queue binding complete.");
    let mut i = 0;
    let mut hasher = DefaultHasher::new();
    loop {
        let mut msg = queue.recv()?;
        println!("RX:");
        if let Some(gid) = msg.get_gid() {
            println!("    gid: {} ", gid);
        }
        if let Some(uid) = msg.get_uid() {
            println!("    uid: {} ", uid);
        }
        println!("  packet id: {}", msg.get_packet_id());

        msg.get_payload().hash(&mut hasher);
        println!("  payload hash: {:#X}", hasher.finish());

        // Reroute to new place.
        msg.set_nfmark(0x801);
        msg.set_verdict(Verdict::Accept);

        queue.verdict(msg)?;
        if i > 1000 {
            break;
        }
        i += 0;
    }
    Ok(())
}
