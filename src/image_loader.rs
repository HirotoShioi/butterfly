use kanaria::UCSStr;
use reqwest::StatusCode;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io;

const HOST_URL: &str = "http://biokite.com/worldbutterfly/";
const DIRECTORY_NAME: &str = "images/";

pub fn get_image(subdir: &str, url: &str) -> Result<String, Box<dyn Error>> {
    let img_url = [HOST_URL, url].concat();
    let mut response = reqwest::get(&img_url)?;

    if response.status() != StatusCode::OK {
        return Err(Box::new(ImageNotFound));
    }

    let fname = response
        .url()
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) });

    match fname {
        None => return Err(Box::new(ImageNameUnknown)),
        Some(name) => {
            let file_path = [DIRECTORY_NAME, subdir, "/", name].concat();
            let file_path = UCSStr::from_str(&file_path).narrow().to_string();
            let mut out = File::create(&file_path)?;
            io::copy(&mut response, &mut out)?;
            Ok(file_path)
        }
    }
}

#[derive(Debug)]
pub enum ImageLoaderError {
    ImageNotFound,
    ImageNameUnknown,
}

impl std::error::Error for ImageLoaderError {}

use ImageLoaderError::*;

impl fmt::Display for ImageLoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Image not found")
    }
}
