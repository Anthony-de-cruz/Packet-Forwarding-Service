use std::{convert::TryFrom, fmt, path::Path};

use image::{DynamicImage, GenericImageView, GrayImage, Luma, imageops::FilterType};
use ndarray::Array4;
use ort::{
    inputs,
    session::{Session, builder::GraphOptimizationLevel},
    value::Value,
};

const INPUT_CHANNELS: usize = 3;
const INPUT_HEIGHT: usize = 224;
const INPUT_WIDTH: usize = 224;
const SESSION_IMAGE_SIDE: usize = 28;
const SESSION_IMAGE_BYTES: usize = SESSION_IMAGE_SIDE * SESSION_IMAGE_SIDE;

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

pub struct Classifier {
    session: Session,
}

impl Classifier {
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
        let tensor = payload_to_tensor(payload)?;
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

fn payload_to_tensor(bytes: &[u8]) -> Result<Array4<f32>, Box<dyn std::error::Error>> {
    if let Ok(image) = image::load_from_memory(bytes) {
        return Ok(preprocess_image(image));
    }

    Ok(preprocess_image(render_bytes_as_image(bytes)))
}

fn render_bytes_as_image(bytes: &[u8]) -> DynamicImage {
    let side = SESSION_IMAGE_SIDE as u32;
    let mut image = GrayImage::new(side, side);

    for index in 0..SESSION_IMAGE_BYTES {
        let value = bytes.get(index).copied().unwrap_or(0);
        let x = (index as u32) % side;
        let y = (index as u32) / side;
        image.put_pixel(x, y, Luma([value]));
    }

    DynamicImage::ImageLuma8(image)
}

fn preprocess_image(image: DynamicImage) -> Array4<f32> {
    let (width, height) = image.dimensions();
    let scale = 256.0 / width.min(height) as f32;
    let resized_width = (width as f32 * scale).round() as u32;
    let resized_height = (height as f32 * scale).round() as u32;
    let image = image.resize_exact(resized_width, resized_height, FilterType::Triangle);

    let x = (resized_width - INPUT_WIDTH as u32) / 2;
    let y = (resized_height - INPUT_HEIGHT as u32) / 2;
    let image = image.crop_imm(x, y, INPUT_WIDTH as u32, INPUT_HEIGHT as u32);
    let image = match image.color().channel_count() {
        1 => DynamicImage::ImageLuma8(image.to_luma8()).to_rgb8(),
        _ => image.to_rgb8(),
    };

    let mut tensor = Array4::<f32>::zeros((1, INPUT_CHANNELS, INPUT_HEIGHT, INPUT_WIDTH));

    for (x, y, pixel) in image.enumerate_pixels() {
        let x = x as usize;
        let y = y as usize;

        tensor[[0, 0, y, x]] = normalize_channel(pixel[0]);
        tensor[[0, 1, y, x]] = normalize_channel(pixel[1]);
        tensor[[0, 2, y, x]] = normalize_channel(pixel[2]);
    }

    tensor
}

fn normalize_channel(value: u8) -> f32 {
    (value as f32 / 255.0 - 0.5) / 0.5
}
