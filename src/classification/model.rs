
use image::{imageops::FilterType, open};
use ndarray::Array4;
use ort::{inputs, session::Session, value::Value};

///
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum TrafficType {
    GoogleMeet = 0,
    Tiktok = 1,
    Instagram = 2,
    Youtube = 3,
    Twitter = 4,
}

/// Run inference on a single image
/// Returns the predicted class index and confidence scores for all classes
pub fn classify_image(
    session: &mut Session,
    image_path: &str,
) -> Result<(usize, Vec<f32>), Box<dyn std::error::Error>> {
    let input_tensor = load_and_preprocess_image(image_path)?;
    let input_value = Value::from_array(input_tensor)?;
    let outputs = session.run(inputs![input_value])?;

    let predictions = outputs[0].try_extract_array::<f32>()?;
    let pred_vec: Vec<f32> = predictions.iter().copied().collect();

    let predicted_class = pred_vec
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(idx, _)| idx)
        .unwrap_or(0);

    Ok((predicted_class, pred_vec))
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
