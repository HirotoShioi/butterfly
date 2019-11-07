//! # butterfly_extractor
//!
//! This crate attempts to extract data from http://biokite.com/worldbutterfly/butterfly-index.htm
//! Data include images, pdfs, both japanese and English name, as well as
//! background color it is being used.
//!
//! We are also using [Google Cloud Vision API](https://cloud.google.com/vision/?hl=ja)
//! to extract colors from the images
//!
//! ## How to start
//!
//! You'd start using this library by defining an instance of `Client` which takes
//! vector of `WebpageParser`.
//!  
//!
//! ```rust
//!     let mut client = Client::new(vec![
//!        WebpageParser::new(
//!            "old_north",
//!            "旧北区",
//!            "http://biokite.com/worldbutterfly/butterfly-PArc.htm#PAall",
//!        ),
//!        WebpageParser::new(
//!            "new_north",
//!            "新北区",
//!            "http://biokite.com/worldbutterfly/butterfly-NArc.htm#NAsa",
//!        ),
//!        WebpageParser::new(
//!            "new_tropical",
//!            "新熱帯区",
//!         "http://biokite.com/worldbutterfly/butterfly-NTro.htm#NTmap",
//!        ),
//!        WebpageParser::new(
//!            "india_australia",
//!            "インド・オーストラリア区",
//!            "http://biokite.com/worldbutterfly/butterfly-IOrs.htm#IOmap",
//!        ),
//!        WebpageParser::new(
//!            "tropical_africa",
//1            "熱帯アフリカ区",
//!            "http://biokite.com/worldbutterfly/butterfly-TAfr.htm#TAmaps",
//!        ),
//!    ]);
//! ```
//!
//! After that, call `collect_data` to start collect data from the webpage. This
//! will return `ButterflyData` struct which can be used to fetch assets such as
//! jpeg images, pdf files, etc.
//!
//! ```rust
//!     let mut butterfly_data = client.collect_datas();
//!
//!    butterfly_data
//!        .fetch_images()
//!        .fetch_pdfs()
//!        .fetch_dominant_colors()
//!        .unwrap();
//! ```
//!
//! After everything is done, call `store_json` to store the data on json file
//!
//! ```rust
//!        butterfly_data
//!            .fetch_images()
//!            .fetch_pdfs()
//!            .fetch_dominant_colors()
//!            .store_json()
//!            .unwrap();
//! ```

extern crate csv;
extern crate env_logger;
extern crate hex;
extern crate kanaria;
extern crate log;
extern crate rayon;
extern crate reqwest;
extern crate scoped_threadpool;
extern crate scraper;
extern crate serde;
extern crate serde_json;

mod butterfly;
mod cloud_vision;
mod constants;
mod errors;
mod webpage_parser;

pub use butterfly::{Butterfly, ButterflyRegion};
pub use cloud_vision::Color;
pub use errors::ButterflyError;
pub use webpage_parser::WebpageParser;

use constants::*;
use log::info;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, remove_dir_all, File};
use std::io;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

///Client used to fetch data from the website
pub struct Client {
    targets: Vec<WebpageParser>,
}

impl Client {
    /// Create an new instance of `Client`
    pub fn new(targets: Vec<WebpageParser>) -> Client {
        Client { targets }
    }

    /// Collect datas from butterfly website
    pub fn collect_datas(&mut self) -> ButterflyData {
        let mut regions = Vec::new();

        for target in self.targets.iter_mut() {
            rayon::scope(|s| {
                s.spawn(|_| {
                    info!("Extracting data from: {}", &target.region);
                    let region = target.fetch_data().unwrap();
                    regions.push(region);
                    info!("Finished extracting data from: {}", &target.region);
                });
            });
        }

        ButterflyData { regions }
    }
}

/// Struct used to collect datas, files such as images, pdfs
pub struct ButterflyData {
    regions: Vec<ButterflyRegion>,
}

impl ButterflyData {
    /// Fetch csv info
    pub fn fetch_csv_info(&mut self) -> &mut Self {
        self.regions.par_iter_mut().for_each(|region| {
            info!("Fetching informations from CSV file");
            region.fetch_csv_info();
            info!(
                "Finished fetching information of region: {}",
                &region.region
            );
        });

        self
    }

    /// Download images from the website and store on `assets` directory
    pub fn fetch_images(&mut self) -> &mut Self {
        self.regions.par_iter_mut().for_each(|region| {
            info!("Fetching images of region: {}", &region.region);
            region.fetch_images();
            info!("Finished dowloading images of region: {}", &region.region);
        });

        self
    }

    /// Download pdf files from the website and store on `assets` directory
    pub fn fetch_pdfs(&mut self) -> &mut Self {
        self.regions.par_iter_mut().for_each(|region| {
            info!("Fetching pdf files of region: {}", &region.region);
            region.fetch_pdfs();
            info!(
                "Finished collecting pdf files of region: {}",
                &region.region
            );
        });

        self
    }

    /// Use Google Cloud Vision to fetch dominant color data
    pub fn fetch_dominant_colors(&mut self) -> &mut Self {
        self.regions.par_iter_mut().for_each(|region| {
            info!("Collecting dominant color data of: {}", &region.region);
            region.fetch_dominant_colors();
            info!(
                "Finished collecting dominant color data of: {}",
                &region.region
            );
        });

        self
    }

    /// Convert the `ButterflyData` into `ButterflyJSON`, then store it on JSON file
    pub fn store_json(&mut self) -> Result<(), io::Error> {
        let mut butterflies = Vec::new();

        let dir_path = Path::new(ASSET_DIRECTORY);

        if create_dir_all(&dir_path).is_err() {
            remove_dir_all(&dir_path)?;
            create_dir_all(&dir_path)?;
        };

        info!(
            "Storing the results to json file on: {}",
            &dir_path.to_str().unwrap()
        );

        let mut butterfly_num: usize = 0;
        let mut pdf_num: usize = 0;

        self.regions.iter().for_each(|region| {
            let mut region_butterflies = region.butterflies.clone();
            pdf_num += region.pdfs.len();
            butterfly_num += region_butterflies.len();
            butterflies.append(&mut region_butterflies);
        });

        // Remove duplicates
        butterflies.sort_by(|b1, b2| b1.jp_name.cmp(&b2.jp_name));
        butterflies.dedup_by(|b1, b2| b1.jp_name == b2.jp_name && b1.eng_name == b2.eng_name);

        let butterfly_json = ButterflyJSON::new(&butterflies, butterfly_num, pdf_num);
        let json_file = File::create(dir_path.join(JSON_FILE_NAME))?;
        serde_json::to_writer_pretty(json_file, &butterfly_json)?;

        Ok(())
    }
}

///Struct used to export data as JSON
#[derive(Deserialize, Serialize, Debug, PartialEq, PartialOrd, Clone)]
pub struct ButterflyJSON {
    /// List of butterflies
    pub butterflies: Vec<Butterfly>,
    /// Number of butterfly data
    pub butterfly_num: usize,
    /// Number of pdf files
    pub pdf_num: usize,
    pub created_at: u64,
}

impl ButterflyJSON {
    fn new(butterflies: &[Butterfly], butterfly_num: usize, pdf_num: usize) -> Self {
        let created_at = now();

        ButterflyJSON {
            butterflies: butterflies.to_owned(),
            butterfly_num,
            pdf_num,
            created_at,
        }
    }
}

fn now() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs()
}
