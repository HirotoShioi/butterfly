use log::info;
use std::fs::File;
use std::io::BufReader;

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
    ///
    ///```rust
    ///let mut client = Client::new(vec![
    ///    WebpageParser::new(
    ///        "old_north",
    ///        "旧北区",
    ///        "http://biokite.com/worldbutterfly/butterfly-PArc.htm#PAall",
    ///    ),
    ///    WebpageParser::new(
    ///        "new_north",
    ///        "新北区",
    ///        "http://biokite.com/worldbutterfly/butterfly-NArc.htm#NAsa",
    ///    ),
    ///    WebpageParser::new(
    ///        "new_tropical",
    ///        "新熱帯区",
    ///        "http://biokite.com/worldbutterfly/butterfly-NTro.htm#NTmap",
    ///    ),
    ///    WebpageParser::new(
    ///        "india_australia",
    ///        "インド・オーストラリア区",
    ///        "http://biokite.com/worldbutterfly/butterfly-IOrs.htm#IOmap",
    ///    ),
    ///    WebpageParser::new(
    ///        "tropical_africa",
    ///        "熱帯アフリカ区",
    ///        "http://biokite.com/worldbutterfly/butterfly-TAfr.htm#TAmaps",
    ///    ),
    ///]);
    ///```
    pub fn new(targets: Vec<WebpageParser>) -> Client {
        Client { targets }
    }

    /// Collect datas from butterfly website
    ///
    ///```rust
    /// let mut client = Client::new(vec![
    ///    WebpageParser::new(
    ///        "old_north",
    ///        "旧北区",
    ///        "http://biokite.com/worldbutterfly/butterfly-PArc.htm#PAall",
    ///    )]);
    /// let result = client.collect_datas.unwrap();
    ///```
    pub fn collect_datas(&mut self) -> Result<ButterflyCollector, ButterflyError> {
        let mut results = Vec::new();

        for target in self.targets.iter_mut() {
            info!("Extracting data from: {}", &target.region);
            let result = target
                .fetch_data()
                .map_err(|_| FailedToFetchHTML(target.url.clone()))?;
            results.push(result.to_owned());
            info!("Finished extracting data from: {}", &target.region);
        }

        ButterflyCollector::from_parse_result(results)
    }

    /// Retrieve data from JSON file
    ///
    /// ```rust
    ///     let result = Client::from_path("path_to_json").unwrap();
    /// ```
    pub fn from_path(json_path: &str) -> Result<ButterflyCollector, ButterflyError> {
        // Open the file in read-only mode with buffer.
        let file =
            File::open(json_path).map_err(|_e| return JsonFileNotFound(json_path.to_string()))?;
        let reader = BufReader::new(file);

        // Read the JSON contents of the file as an instance of `User`.
        let butterfly_json: ButterflyJSON = serde_json::from_reader(reader)
            .map_err(|_f| return FailedToParseJson(json_path.to_string()))?;

        butterfly_json.into_collector()
    }
}
