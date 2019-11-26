use thiserror::Error;

/// List of errors that could occur when processing data
#[derive(Debug, Error)]
pub enum ButterflyError {
    /// Image source was not found when extracting
    #[error("Image source not found when parsing the page")]
    ImageSourceNotFound,
    /// Text data was not found when extracting
    #[error("Text description of a butterfly could not be extracted")]
    TextNotFound,
    /// Butterfly was not found
    #[error("Index of given butterfly does not exist")]
    InvalidIndexButterflyNotFound,
    /// Failed to fetch html data
    #[error("Failed to fetch html: {0}")]
    FailedToFetchHTML(String),
    /// Name of the image is unknown
    #[error("Image name unknown")]
    ImageNameUnknown,
    /// Failed to parse CSV file
    #[error("Failed to parse CSV record")]
    FailedToParseCSVRecord(String),
    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(String),
    /// File name was unknown
    #[error("File name unknown")]
    FileNameUnknown,
    /// JSON file not found
    #[error("JSON file not found: {0}")]
    JsonFileNotFound(String),
    /// Failed to parse JSON file
    #[error("Failed to parse JSON file: {0}")]
    FailedToParseJson(String),
}
