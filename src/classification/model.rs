use std::{convert::TryFrom, fmt, path::Path};

use image::{GrayImage, imageops, imageops::FilterType};
use ndarray::Array4;
use ort::{
    inputs,
    session::{Session, builder::GraphOptimizationLevel},
    value::Value,
};

const INPUT_CHANNELS: u32 = 3;
const INPUT_HEIGHT: u32 = 224;
const INPUT_WIDTH: u32 = 224;
const SESSION_IMAGE_SIDE: u32 = 28;
const SESSION_IMAGE_BYTES: u32 = SESSION_IMAGE_SIDE * SESSION_IMAGE_SIDE;

/// Matches with CNN models classifications.
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
    //pub scores: Vec<f32>,
}

/// Represents a CNN with runtime for inference.
pub struct Classifier {
    session: Session,
}

impl Classifier {
    /// Load a classifier from an exported ONNX model file.
    ///
    /// # Arguments
    ///
    /// * `model_path`: Path to the ONNX model produced by the training pipeline.
    ///
    /// # Errors
    ///
    /// Returns an error if the runtime cannot be created.
    pub fn from_file(model_path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let model_path = model_path.as_ref();
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::All)?
            .commit_from_file(model_path)?;

        Ok(Self { session })
    }

    /// Classify a single packet payload.
    ///
    /// # Arguments
    ///
    /// * `payload`: Raw packet payload bytes. Short payloads are zero-padded and
    ///   longer payloads are truncated to the fixed model input window.
    ///
    /// # Errors
    ///
    /// Returns an error if tensor creation succeeds but model execution or
    /// output extraction fails.
    pub fn classify_payload(
        &mut self,
        payload: &[u8],
    ) -> Result<Classification, Box<dyn std::error::Error>> {
        let tensor = payload_to_tensor(payload);
        let input_value = Value::from_array(tensor)?;
        let outputs = self.session.run(inputs![input_value])?;
        let predictions = outputs[0].try_extract_array::<f32>()?;
        let scores: Vec<f32> = predictions.iter().copied().collect();
        let traffic_type = predicted_traffic_type(&scores);

        Ok(Classification {
            traffic_type,
            //scores,
        })
    }
}

/// Pick the highest-scoring traffic class from a model output tensor.
///
/// # Arguments
///
/// * `scores`: The score set produced by the CNN.
///
/// # Panics
///
/// Panics if the model returns no scores or if the winning index does not map
/// to a known [`TrafficType`].
fn predicted_traffic_type(scores: &[f32]) -> TrafficType {
    let predicted_index = scores
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .map_or_else(
            || panic!("model returned no prediction scores"),
            |(index, _)| index,
        );

    TrafficType::try_from(predicted_index)
        .unwrap_or_else(|()| panic!("model returned unknown class index {predicted_index}"))
}

/// Convert the payload into a usable tensor.
/// This preserves the model's training-time resize/crop behaviour.
///
/// # Arguments
///
/// * `bytes`: The packet payload bytes.
///
/// # Returns
///
/// A tensor with shape `(1, 3, 224, 224)`.
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
fn payload_to_tensor(bytes: &[u8]) -> Array4<f32> {
    let mut pixels = vec![0_u8; SESSION_IMAGE_BYTES as usize];
    let copied_len = bytes.len().min(pixels.len());
    pixels[..copied_len].copy_from_slice(&bytes[..copied_len]);

    let image = GrayImage::from_raw(SESSION_IMAGE_SIDE, SESSION_IMAGE_SIDE, pixels)
        .expect("grayscale payload buffer dimensions should always match");
    let scale = 256.0 / SESSION_IMAGE_SIDE as f32;
    let resized_side = (SESSION_IMAGE_SIDE as f32 * scale).round() as u32;
    let image = imageops::resize(&image, resized_side, resized_side, FilterType::Triangle);

    let x = (resized_side - INPUT_WIDTH) / 2;
    let y = (resized_side - INPUT_HEIGHT) / 2;
    let image = imageops::crop_imm(&image, x, y, INPUT_WIDTH, INPUT_HEIGHT).to_image();

    let mut tensor = Array4::<f32>::zeros((
        1,
        INPUT_CHANNELS as usize,
        INPUT_HEIGHT as usize,
        INPUT_WIDTH as usize,
    ));

    for (x, y, pixel) in image.enumerate_pixels() {
        // Normalise.
        let value = (f32::from(pixel[0]) / 255.0 - 0.5) / 0.5;

        // The model expects three channels, but the source payload is grayscale.
        let x = x as usize;
        let y = y as usize;
        tensor[[0, 0, y, x]] = value;
        tensor[[0, 1, y, x]] = value;
        tensor[[0, 2, y, x]] = value;
    }

    tensor
}
