use std::fmt;

use super::errors::ButterflyRegionError::*;

#[derive(Debug)]
pub enum ButterflyRegionError {
    ImageSourceNotFound,
    TextNotFound,
    InvalidIndexButterflyNotFound,
    FailedToFetchHTML,
    ImageNotFound,
    ImageNameUnknown,
    NotImage,
}

impl std::error::Error for ButterflyRegionError {}

impl fmt::Display for ButterflyRegionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        let error_message = match self {
            ImageSourceNotFound => "Image source not found",
            FailedToFetchHTML => "Failed to fetch html",
            InvalidIndexButterflyNotFound => "Index of given butterfly does not exist",
            TextNotFound => "Text description of a butterfly could not be extracted",
            ImageNotFound => "Image could not be fetched",
            ImageNameUnknown => "Image name unknown",
            NotImage => "Downloaded file is not image file",
        };
        write!(f, "{}", error_message)
    }
}
