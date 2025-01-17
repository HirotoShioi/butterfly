/// Url of the website
pub const BUTTERFLY_URL: &str = "http://biokite.com/worldbutterfly/";
/// Directory which stores the downloaded files
pub const ASSET_DIRECTORY: &str = "./assets";
/// Directory which stores the images
pub const IMAGE_DIRECTORY: &str = "images";
/// Directory which store the pdf files
pub const PDF_DIRECTORY: &str = "pdf";
/// Number of threads used for fetching google cloud vision api
pub const GCV_THEAD_POOL_NUM: u32 = 30;
/// Path to CSV file
pub const CSV_FILE_PATH: &str = "./butterfly.csv";
/// Google Cloud Vision API
pub const CLOUD_VISION_URI: &str = "https://vision.googleapis.com/v1/images:annotate";
/// Path to API key
pub const API_KEY_FILE_PATH: &str = "./secrets/vision_api.key";
