mod model;
mod server;

use std::{
    collections::HashMap,
    io::Write,
    net::{SocketAddr, TcpListener, TcpStream},
    str::FromStr,
};

use model::model::TrafficType;
use server::server::redirect;

struct ForwardingSocket {
    addr: SocketAddr,
    stream: TcpStream,
}

fn main() {
    println!("Start");
    redirect().unwrap();
    let input_addr_str = "127.0.0.1:8999";
    let node_0_addr_str = "127.0.0.1:8000";
    let node_1_addr_str = "127.0.0.1:8001";
    let node_2_addr_str = "127.0.0.1:8002";
    let node_3_addr_str = "127.0.0.1:8003";

    println!("Listening on {input_addr_str}");
    println!(
        r#"Fowarding to nodes:
    0: {node_0_addr_str}
    1: {node_1_addr_str}
    2: {node_2_addr_str}
    3: {node_3_addr_str}"#
    );

    let input_addr = SocketAddr::from_str(input_addr_str).unwrap();
    let node_0_addr = SocketAddr::from_str(node_0_addr_str).unwrap();
    let node_1_addr = SocketAddr::from_str(node_1_addr_str).unwrap();
    let node_2_addr = SocketAddr::from_str(node_2_addr_str).unwrap();
    let node_3_addr = SocketAddr::from_str(node_3_addr_str).unwrap();

    let mut route_map = HashMap::new();
    route_map.insert(TrafficType::GoogleMeet, node_0_addr);

    let mut node_0_stream = TcpStream::connect(node_0_addr).unwrap();

    let mut node_streams = HashMap::new();
    node_streams.insert(node_0_addr, &mut node_0_stream);

    let listener = TcpListener::bind(input_addr).unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => match consume_bytes(stream) {
                Ok(buff) => {
                    let stream = node_streams.get_mut(&node_0_addr).unwrap();
                    stream.write_all(&buff).unwrap();
                }
                Err(err) => {
                    eprintln!("TCP stream error: {err}")
                }
            },
            Err(err) => {
                eprintln!("TCP stream error: {err}");
            }
        }
    }

    println!("Server shutting down");
}
