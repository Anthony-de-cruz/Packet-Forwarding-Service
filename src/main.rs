mod classification;
mod server;

use crate::classification::model::Classifier;

use server::redirect;

const MODEL_PATH: &str = "Packet-Classifier/out/model.onnx";

fn open_classifier() -> Result<Classifier, Box<dyn std::error::Error>> {
    println!("Loading traffic classifier from {MODEL_PATH}...");
    let classifier = Classifier::from_file(MODEL_PATH)?;
    println!("Traffic classifier ready.");
    Ok(classifier)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut classifier = open_classifier()?;
    redirect(&mut classifier)
}
