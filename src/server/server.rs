use nfq::{Queue, Verdict};

pub fn redirect() -> std::io::Result<()> {
    println!("opening queue...");
    let mut queue = Queue::open()?;
    println!("queue opened?");
    queue.bind(0)?;
    println!("bind complete?");
    let mut i = 0;
    loop {
        println!("awaiting msg");
        let mut msg = queue.recv()?;
        println!("woaah");
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

        i += 1;
        queue.verdict(msg)?;
    }
    //Ok(())
}

