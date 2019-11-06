extern crate base64;

use hex;
use reqwest::{StatusCode, Url};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fmt;
use std::fs;

const CLOUD_VISION_URI: &str = "https://vision.googleapis.com/v1/images:annotate";
const API_KEY_FILE_PATH: &str = "./secrets/vision_api.key";

/// Get list of `Color` using Google Cloud Vision API
pub fn get_dominant_colors(image_url: &Url) -> Result<Vec<Color>, CloudVisionError> {
    let response_json = use_cloud_vision_api(image_url)?;
    let extracted_color_vec = extract_colors(&response_json)?;

    if extracted_color_vec.is_empty() {
        return Err(VectorIsEmpty);
    }

    Ok(extracted_color_vec)
}

/// Use cloud vision api
fn use_cloud_vision_api(image_url: &Url) -> Result<Value, CloudVisionError> {
    let base64_image =
        get_base64_image(image_url).ok_or_else(|| UnableToFetchImage(image_url.to_owned()))?;

    let request = json!({
        "requests": [
          {
            "image": {
                "content": base64_image
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

    let secret_key = fs::read_to_string(API_KEY_FILE_PATH).unwrap();

    let mut response = reqwest::Client::new()
        .post(CLOUD_VISION_URI)
        .query(&[("key", secret_key)])
        .json(&request)
        .send()
        .map_err(|_err| FailedGCV)?;

    if response.status() != StatusCode::OK {
        return Err(BadRequest(image_url.to_owned()));
    }

    let response_json: Value = response.json().map_err(|_err| NotJSON)?;

    let err = &response_json["responses"][0]["error"];

    if err.is_object() {
        if let Some(error_message) = &err["message"].as_str() {
            return Err(FailedToParseImage((*error_message).to_string()));
        } else {
            return Err(UnknownError);
        };
    }

    Ok(response_json)
}

/// Extract `Vec<Color>` with given `Value`
fn extract_colors(val: &Value) -> Result<Vec<Color>, CloudVisionError> {
    let colors = &val["responses"][0]["imagePropertiesAnnotation"]["dominantColors"]["colors"];

    match colors.as_array() {
        Some(color_ary) => {
            let mut color_vec = Vec::new();

            color_ary.iter().for_each(|color_value| {
                if let Some(color) = to_color(color_value) {
                    color_vec.push(color);
                };
            });
            Ok(color_vec)
        }

        None => Err(UnableToParseColorData(val.to_owned())),
    }
}

/// Color struct
#[derive(Debug, PartialEq, PartialOrd, Clone, Serialize, Deserialize)]
pub struct Color {
    /// Pixel fraction
    pub pixel_fraction: f32,
    /// Score
    pub score: f32,
    /// Color in hex string
    pub hex_color: String,
}

/// Construct `Color` struct with given `Value`
fn to_color(value: &Value) -> Option<Color> {
    let pixel_fraction = value.get("pixelFraction")?.as_f64()? as f32;
    let score = value.get("score")?.as_f64()? as f32;

    let color = &value.get("color")?;

    // For unknown reason, some responsones does not have all the fields. WIP
    let red: u8 = color.get("red")?.to_owned().as_u64()? as u8;
    let green: u8 = color.get("green")?.to_owned().as_u64()? as u8;
    let blue: u8 = color.get("blue")?.to_owned().as_u64()? as u8;

    let hex = hex::encode(vec![red, green, blue]);

    let hex_color = format!("#{}", hex);

    let color_struct = Color {
        pixel_fraction,
        score,
        hex_color,
    };

    Some(color_struct)
}

/// Get image content and encod it with base64
fn get_base64_image(image_url: &Url) -> Option<String> {
    let mut response = reqwest::get(image_url.to_owned()).ok()?;

    if response.status() != StatusCode::OK {
        return None;
    }

    let mut buf: Vec<u8> = vec![];
    response.copy_to(&mut buf).ok()?;
    let encoded = base64::encode(&buf);
    Some(encoded)
}

use super::cloud_vision::CloudVisionError::*;

#[derive(Debug, PartialEq, Clone)]
pub enum CloudVisionError {
    UnableToFetchImage(Url),
    BadRequest(Url),
    FailedToParseImage(String),
    UnableToParseColorData(Value),
    UnknownError,
    VectorIsEmpty,
    FailedGCV,
    NotJSON,
}

impl std::error::Error for CloudVisionError {}

impl fmt::Display for CloudVisionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        let error_message: String = match self {
            BadRequest(url) => format!("Bad request: {}", url),
            FailedToParseImage(msg) => format!("Cloud vision api failed to parse image: {}", msg),
            UnableToParseColorData(val) => format!("Unable to parse data: {:#?}", val),
            UnknownError => String::from("Unknown error"),
            VectorIsEmpty => String::from("Extracted data is empty"),
            FailedGCV => String::from("Failed to request Google Cloud Vision API"),
            UnableToFetchImage(url) => format!("Unable to fetch image from url: {}", url),
            NotJSON => String::from("Response body is not JSON"),
        };
        write!(f, "{}", error_message)
    }
}
