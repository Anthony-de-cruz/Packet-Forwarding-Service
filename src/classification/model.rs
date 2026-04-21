use std::{convert::TryFrom, fmt, path::Path};

use ndarray::Array4;
use ort::{
    inputs,
    session::{Session, builder::GraphOptimizationLevel},
    value::Value,
};

const INPUT_CHANNELS: usize = 3;
const INPUT_HEIGHT: usize = 224;
const INPUT_WIDTH: usize = 224;
const INPUT_SIZE: usize = INPUT_CHANNELS * INPUT_HEIGHT * INPUT_WIDTH;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TrafficType {
    GoogleMeet = 0,
    Instagram = 1,
    TikTok = 2,
    Twitter = 3,
    Youtube = 4,
}

impl TryFrom<usize> for TrafficType {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            x if x == TrafficType::GoogleMeet as usize => Ok(TrafficType::GoogleMeet),
            x if x == TrafficType::Instagram as usize => Ok(TrafficType::Instagram),
            x if x == TrafficType::TikTok as usize => Ok(TrafficType::TikTok),
            x if x == TrafficType::Twitter as usize => Ok(TrafficType::Twitter),
            x if x == TrafficType::Youtube as usize => Ok(TrafficType::Youtube),
            _ => Err(()),
        }
    }
}

impl fmt::Display for TrafficType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TrafficType::GoogleMeet => write!(formatter, "Google Meet"),
            TrafficType::Instagram => write!(formatter, "Instagram"),
            TrafficType::TikTok => write!(formatter, "TikTok"),
            TrafficType::Twitter => write!(formatter, "Twitter"),
            TrafficType::Youtube => write!(formatter, "YouTube"),
        }
    }
}

pub struct Classification {
    pub traffic_type: TrafficType,
    pub scores: Vec<f32>,
}

pub struct TrafficClassifier {
    session: Session,
}

impl TrafficClassifier {
    pub fn from_file(model_path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let model_path = model_path.as_ref();
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::All)?
            .commit_from_file(model_path)?;

        Ok(Self { session })
    }

    pub fn classify_payload(
        &mut self,
        payload: &[u8],
    ) -> Result<Classification, Box<dyn std::error::Error>> {
        let tensor = packet_bytes_to_tensor(payload);
        let input_value = Value::from_array(tensor)?;
        let outputs = self.session.run(inputs![input_value])?;
        let predictions = outputs[0].try_extract_array::<f32>()?;
        let scores: Vec<f32> = predictions.iter().copied().collect();
        let traffic_type = predicted_traffic_type(&scores)?;

        Ok(Classification {
            traffic_type,
            scores,
        })
    }
}

fn predicted_traffic_type(scores: &[f32]) -> Result<TrafficType, Box<dyn std::error::Error>> {
    let predicted_index = scores
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(index, _)| index)
        .ok_or("model returned no prediction scores")?;

    TrafficType::try_from(predicted_index)
        .map_err(|_| format!("model returned unknown class index {predicted_index}").into())
}

fn packet_bytes_to_tensor(bytes: &[u8]) -> Array4<f32> {
    let mut tensor = Array4::<f32>::zeros((1, INPUT_CHANNELS, INPUT_HEIGHT, INPUT_WIDTH));

    for index in 0..INPUT_SIZE {
        let value = bytes.get(index).copied().unwrap_or(0) as f32;
        let channel = index / (INPUT_HEIGHT * INPUT_WIDTH);
        let pixel_index = index % (INPUT_HEIGHT * INPUT_WIDTH);
        let y = pixel_index / INPUT_WIDTH;
        let x = pixel_index % INPUT_WIDTH;

        tensor[[0, channel, y, x]] = (value / 255.0 - 0.5) / 0.5;
    }

    tensor
}
