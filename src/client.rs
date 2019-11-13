use log::info;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use super::butterfly_collector::{ButterflyCollector, ButterflyJSON};
use super::errors::ButterflyError::{self, *};
use super::webpage_parser::WebpageParser;

/// Client used to retrieve butterfly data
/// 
/// You can retrieve data from the Website using `new` then calling `collect_datas`
/// 
/// You can also retrieve data from JSON file with `from_path`
pub struct Client {
    targets: Vec<WebpageParser>,
}

impl Client {
    /// Create an new instance of `Client`
    pub fn new(targets: Vec<WebpageParser>) -> Client {
        Client { targets }
    }

    /// Collect datas from butterfly website
    pub fn collect_datas(&mut self) -> Result<ButterflyCollector, ButterflyError> {
        let mut results = Vec::new();

        for target in self.targets.iter_mut() {
            info!("Extracting data from: {}", &target.region);
            let result = target.fetch_data().map_err(|_| FailedToFetchHTML)?;
            results.push(result.to_owned());
            info!("Finished extracting data from: {}", &target.region);
        }

        ButterflyCollector::from_parse_result(results)
    }


    /// Retrieve data from JSON file
    pub fn from_path<P: AsRef<Path>>(json_path: P) -> Result<ButterflyCollector, ButterflyError> {
        // Open the file in read-only mode with buffer.
        let file = File::open(json_path).map_err(|_| return JsonFileNotFound)?;
        let reader = BufReader::new(file);

        // Read the JSON contents of the file as an instance of `User`.
        let butterfly_json: ButterflyJSON =
            serde_json::from_reader(reader).map_err(|_f| return FailedToParseJson)?;

        butterfly_json.into_collector()
    }
}
