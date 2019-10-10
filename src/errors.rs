use std::fmt;

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
            ButterflyRegionError::ImageSourceNotFound => "Image source not found",
            ButterflyRegionError::FailedToFetchHTML => "Failed to fetch html",
            ButterflyRegionError::InvalidIndexButterflyNotFound => {
                "Index of given butterfly does not exist"
            }
            ButterflyRegionError::TextNotFound => {
                "Text description of a butterfly could not be extracted"
            }
            ButterflyRegionError::ImageNotFound => "Image could not be fetched",
            ButterflyRegionError::ImageNameUnknown => "Image name unknown",
            ButterflyRegionError::NotImage => "Downloaded file is not image file",
        };
        write!(f, "{}", error_message)
    }
}
