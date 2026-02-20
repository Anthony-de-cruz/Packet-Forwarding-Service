use std::{convert::TryFrom, fmt};

use image::{DynamicImage, GenericImageView, imageops::FilterType, open};
use ndarray::Array4;
use ort::{inputs, session::Session, value::Value};

///
#[derive(Debug, PartialEq, Eq, Hash)]
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
            TrafficType::TikTok => write!(formatter, "Tiktok"),
            TrafficType::Twitter => write!(formatter, "Twitter"),
            TrafficType::Youtube => write!(formatter, "Youtube"),
        }
    }
}

pub fn classify_tensor(
    session: &mut Session,
    tensor: Array4<f32>,
) -> Result<TrafficType, Box<dyn std::error::Error>> {
    let input_value = Value::from_array(tensor)?;

    let outputs = session.run(inputs![input_value])?;

    let predictions = outputs[0].try_extract_array::<f32>()?;
    let prediction_vec: Vec<f32> = predictions.iter().copied().collect();

    let predicted_type = TrafficType::try_from(
        prediction_vec
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(idx, _)| idx)
            .unwrap_or(0),
    )
    .unwrap();

    Ok(predicted_type)
}

/// Run inference on a single image
/// Returns the predicted class index and confidence scores for all classes
pub fn classify_image(
    session: &mut Session,
    image_path: &str,
) -> Result<(TrafficType, Vec<f32>), Box<dyn std::error::Error>> {
    let input_tensor = load_and_preprocess_image_2(image_path)?;
    let input_value = Value::from_array(input_tensor)?;
    let outputs = session.run(inputs![input_value])?;

    let predictions = outputs[0].try_extract_array::<f32>()?;
    let prediction_vec: Vec<f32> = predictions.iter().copied().collect();

    let predicted_type = TrafficType::try_from(
        prediction_vec
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(idx, _)| idx)
            .unwrap_or(0),
    )
    .unwrap();

    Ok((predicted_type, prediction_vec))
}

/// Load an image file and preprocess it for ResNet50
/// Input: path to image file
/// Output: 4D tensor (1, 3, 224, 224) normalized with mean=0.5, std=0.5
fn load_and_preprocess_image(image_path: &str) -> Result<Array4<f32>, Box<dyn std::error::Error>> {
    let img = open(image_path)?;
    let img = img.resize_exact(224, 224, FilterType::Lanczos3);
    let img_rgb = img.to_rgb8();

    let mut tensor = Array4::<f32>::zeros((1, 3, 224, 224));

    for (x, y, pixel) in img_rgb.enumerate_pixels() {
        let [r, g, b] = [pixel[0] as f32, pixel[1] as f32, pixel[2] as f32];
        tensor[[0, 0, y as usize, x as usize]] = (r / 255.0 - 0.5) / 0.5;
        tensor[[0, 1, y as usize, x as usize]] = (g / 255.0 - 0.5) / 0.5;
        tensor[[0, 2, y as usize, x as usize]] = (b / 255.0 - 0.5) / 0.5;
    }

    Ok(tensor)
}

/// Load an image file and preprocess it for ResNet50
/// Matches Python preprocessing:
/// - Resize shortest side to 256
/// - Center crop to 224x224
/// - Convert grayscale to RGB if needed
/// - Normalize to [-1, 1] using mean=0.5, std=0.5
/// Output: 4D tensor (1, 3, 224, 224)
fn load_and_preprocess_image_2(image_path: &str) -> Result<Array4<f32>, Box<dyn std::error::Error>> {
    // Load image
    let img = image::open(image_path)?;

    // Resize so shortest side = 256 while keeping aspect ratio
    let (width, height) = img.dimensions();
    let scale = 256.0 / (width.min(height) as f32);
    let new_width = (width as f32 * scale).round() as u32;
    let new_height = (height as f32 * scale).round() as u32;
    let img = img.resize_exact(new_width, new_height, FilterType::Triangle);

    // Center crop to 224x224
    let x0 = (new_width - 224) / 2;
    let y0 = (new_height - 224) / 2;
    let img = img.crop_imm(x0, y0, 224, 224);

    // Convert to RGB if grayscale
    let img_rgb = match img.color().channel_count() {
        1 => DynamicImage::ImageLuma8(img.to_luma8()).to_rgb8(),
        _ => img.to_rgb8(),
    };

    // Allocate tensor: (1, 3, 224, 224)
    let mut tensor = Array4::<f32>::zeros((1, 3, 224, 224));

    // Fill tensor and normalize to [-1, 1]
    for (x, y, pixel) in img_rgb.enumerate_pixels() {
        let r = pixel[0] as f32;
        let g = pixel[1] as f32;
        let b = pixel[2] as f32;

        tensor[[0, 0, y as usize, x as usize]] = (r / 255.0 - 0.5) / 0.5;
        tensor[[0, 1, y as usize, x as usize]] = (g / 255.0 - 0.5) / 0.5;
        tensor[[0, 2, y as usize, x as usize]] = (b / 255.0 - 0.5) / 0.5;
    }

    Ok(tensor)
}
