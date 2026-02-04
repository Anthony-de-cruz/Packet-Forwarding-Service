use nfq::{Queue, Verdict};

pub fn redirect() -> std::io::Result<()> {
    println!("Opening net filter queue...");
    let mut queue = Queue::open()?;
    println!("Binding net filter queue to interface...");
    queue.bind(0)?;
    let mut i = 0;
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

        println!(" payload: {:?}", msg.get_payload());

        if i % 2 == 1 {
            msg.set_verdict(Verdict::Stop);
        } else {
            msg.set_verdict(Verdict::Accept);
        }

        queue.verdict(msg)?;
        if i > 100 {
            break;
        }
        i += 1;
    }
    Ok(())
}
