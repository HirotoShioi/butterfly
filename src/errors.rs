use std::fmt;

use super::errors::ButterflyError::*;

/// List of errors that could occur when processing data
#[derive(Debug)]
pub enum ButterflyError {
    /// Image source was not found when extracting
    ImageSourceNotFound,
    /// Text data was not found when extracting
    TextNotFound,
    /// Butterfly was not found
    InvalidIndexButterflyNotFound,
    /// Failed to fetch html data
    FailedToFetchHTML,
    /// Image was not found
    ImageNotFound,
    /// Name of the image is unknown
    ImageNameUnknown,
    /// Given data is not a image file
    NotImage,
    ///
    FailedToParseCSVRecord,
}

impl std::error::Error for ButterflyError {}

impl fmt::Display for ButterflyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        let error_message = match self {
            ImageSourceNotFound => "Image source not found",
            FailedToFetchHTML => "Failed to fetch html",
            InvalidIndexButterflyNotFound => "Index of given butterfly does not exist",
            TextNotFound => "Text description of a butterfly could not be extracted",
            ImageNotFound => "Image could not be fetched",
            ImageNameUnknown => "Image name unknown",
            NotImage => "Downloaded file is not image file",
            FailedToParseCSVRecord => "Failed to parse CSV record",
        };
        write!(f, "{}", error_message)
    }
}
