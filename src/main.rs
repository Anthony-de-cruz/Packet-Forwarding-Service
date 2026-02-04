mod classification;
mod server;

use std::fs;

use classification::model::{TrafficType, classify_image};

use ort::session::{Session, builder::GraphOptimizationLevel};

const MODEL_PATH: &str = "model/resnet50_classifier.onnx";

fn main() {
    // Initialize ONNX Runtime
    println!("Loading ONNX model @ {}...", MODEL_PATH);
    let mut session = Session::builder()
        .unwrap()
        .with_optimization_level(GraphOptimizationLevel::All)
        .unwrap()
        .commit_from_file(MODEL_PATH)
        .unwrap();
    println!("Model loaded successfully!");

    for path in fs::read_dir("IG").unwrap() {
        let full_path = format!("{}", path.unwrap().path().display()).to_string();
        println!("\nImage: {}", full_path);
        match classify_image(&mut session, &full_path) {
            Ok((predicted_class, scores)) => {
                //println!("\nImage: {}", image_path);
                println!("Predicted class: {}", predicted_class);
                println!("Confidence scores:");
                for (i, score) in scores.iter().enumerate() {
                    println!("  Class {}: {:.4}", i, score);
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}
//
// fn main() {
//     println!("Start");
//     redirect().unwrap();
//     let input_addr_str = "127.0.0.1:8999";
//     let node_0_addr_str = "127.0.0.1:8000";
//     let node_1_addr_str = "127.0.0.1:8001";
//     let node_2_addr_str = "127.0.0.1:8002";
//     let node_3_addr_str = "127.0.0.1:8003";
//
//     println!("Listening on {input_addr_str}");
//     println!(
//         r#"Forwarding to nodes:
//     0: {node_0_addr_str}
//     1: {node_1_addr_str}
//     2: {node_2_addr_str}
//     3: {node_3_addr_str}"#
//     );
//
//     let input_addr = SocketAddr::from_str(input_addr_str).unwrap();
//     let node_0_addr = SocketAddr::from_str(node_0_addr_str).unwrap();
//     let node_1_addr = SocketAddr::from_str(node_1_addr_str).unwrap();
//     let node_2_addr = SocketAddr::from_str(node_2_addr_str).unwrap();
//     let node_3_addr = SocketAddr::from_str(node_3_addr_str).unwrap();
//
//     let mut route_map = HashMap::new();
//     route_map.insert(TrafficType::GoogleMeet, node_0_addr);
//
//     let mut node_0_stream = TcpStream::connect(node_0_addr).unwrap();
//
//     let mut node_streams = HashMap::new();
//     node_streams.insert(node_0_addr, &mut node_0_stream);
//
//     let listener = TcpListener::bind(input_addr).unwrap();
//     for stream in listener.incoming() {
//         match stream {
//             Ok(stream) => match consume_bytes(stream) {
//                 Ok(buff) => {
//                     let stream = node_streams.get_mut(&node_0_addr).unwrap();
//                     stream.write_all(&buff).unwrap();
//                 }
//                 Err(err) => {
//                     eprintln!("TCP stream error: {err}")
//                 }
//             },
//             Err(err) => {
//                 eprintln!("TCP stream error: {err}");
//             }
//         }
//     }
//
//     println!("Server shutting down");
// }
