use hex;
use reqwest::{StatusCode, Url};
use serde_json::{json, Value};
use std::fmt;
use std::fs;

const CLOUD_VISION_URI: &str = "https://vision.googleapis.com/v1/images:annotate";
const API_KEY_FILE_PATH: &str = "./secrets/vision_api.key";

pub fn get_dominant_colors(image_url: &Url) -> Result<Vec<Color>, Box<dyn std::error::Error>> {
    let response_json = use_cloud_vision_api(image_url)?;
    let extracted = extract_colors(&response_json)?;
    Ok(extracted)
}

fn use_cloud_vision_api(image_url: &Url) -> Result<Value, Box<dyn std::error::Error>> {
    let request = json!({
        "requests": [
          {
            "image": {
                "source": {
                    "imageUri": image_url.to_owned().into_string()
                }
            },
            "features": [
              {
                "maxResults": 10,
                "type": "IMAGE_PROPERTIES"
              }
            ]
          }
        ]
    });

    let secret_key = fs::read_to_string(API_KEY_FILE_PATH)?;

    let mut response = reqwest::Client::new()
        .post(CLOUD_VISION_URI)
        .query(&[("key", secret_key)])
        .json(&request)
        .send()?;

    if response.status() != StatusCode::OK {
        return Err(Box::new(CloudVisionError::BadRequest));
    }

    let response_json: Value = response.json()?;

    let err = &response_json["responses"][0]["error"];

    if err.is_object() {
        return Err(Box::new(CloudVisionError::FailedToParseImage));
    }

    Ok(response_json)
}

fn extract_colors(val: &Value) -> Result<Vec<Color>, CloudVisionError> {
    let colors = &val["responses"][0]["imagePropertiesAnnotation"]["dominantColors"]["colors"];

    match colors.as_array() {
        Some(color_ary) => {
            let mut color_vec = Vec::new();

            for color_value in color_ary.iter() {
                if let Some(color) = to_color(color_value) {
                    color_vec.push(color);
                } else {
                    return Err(CloudVisionError::UnableToParseColorData);
                };
            }
            Ok(color_vec)
        }

        None => Err(CloudVisionError::UnableToParseColorData),
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Color {
    pub pixel_fraction: f32,
    pub score: f32,
    pub hex_color: String,
}

#[derive(Debug)]
enum CloudVisionError {
    BadRequest,
    UnableToParseColorData,
    FailedToParseImage,
}

impl std::error::Error for CloudVisionError {}

impl fmt::Display for CloudVisionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        let error_message = match self {
            CloudVisionError::BadRequest => "Bad request",
            CloudVisionError::FailedToParseImage => "Cloud vision api failed to parse image",
            CloudVisionError::UnableToParseColorData => "Unable to parse data",
        };
        write!(f, "{}", error_message)
    }
}

//Construct `Color` struct with given `Value`
fn to_color(value: &Value) -> Option<Color> {
    let pixel_fraction = value.get("pixelFraction")?.as_f64()? as f32;
    let score = value.get("score")?.as_f64()? as f32;

    let color = &value.get("color")?;
    let red: u8 = color.get("red")?.to_owned().as_u64()? as u8;
    let green: u8 = color.get("green")?.to_owned().as_u64()? as u8;
    let blue: u8 = color.get("blue")?.to_owned().as_u64()? as u8;

    //Construct hex string color
    let mut hex_color = String::from("#");

    let hex = hex::encode(vec![red, green, blue]);

    hex_color.push_str(&hex);

    let color_struct = Color {
        pixel_fraction,
        score,
        hex_color,
    };

    Some(color_struct)
}
