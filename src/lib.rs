extern crate hex;
extern crate kanaria;
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
use std::fs::{create_dir_all, remove_dir_all, File};
use std::io;
use std::path::Path;
use std::time::SystemTime;

///Client used to fetch data from the website
pub struct Client {
    targets: Vec<WebpageParser>,
    pool: scoped_threadpool::Pool,
}

impl Client {
    pub fn new(targets: Vec<WebpageParser>) -> Client {
        let pool = scoped_threadpool::Pool::new(CLIENT_POOL_NUM);
        Client { targets, pool }
    }

    pub fn fetch_datas(&mut self) -> ButterflyData {
        let mut regions = Vec::new();

        for target in self.targets.iter_mut() {
            //Proper exception handling
            self.pool.scoped(|scoped| {
                scoped.execute(|| {
                    let region = target.fetch_data().unwrap();
                    regions.push(region);
                });
            });
        }

        let pool = scoped_threadpool::Pool::new(CLIENT_POOL_NUM);

        ButterflyData { regions, pool }
    }
}

/// Struct used to collect datas, files such as images, pdfs
pub struct ButterflyData {
    regions: Vec<ButterflyRegion>,
    pool: scoped_threadpool::Pool,
}

impl ButterflyData {
    /// Download images from the website and store on `assets` directory
    pub fn fetch_images(&mut self) -> &mut Self {
        for region in self.regions.iter_mut() {
            self.pool.scoped(|scoped| {
                scoped.execute(|| {
                    region.fetch_images();
                });
            });
        }

        self
    }

    /// Download pdf files from the website and store on `assets` directory
    pub fn fetch_pdfs(&mut self) -> &mut Self {
        for region in self.regions.iter_mut() {
            self.pool.scoped(|scoped| {
                scoped.execute(|| {
                    region.fetch_pdfs();
                });
            });
        }

        self
    }

    /// Use Google Cloud Vision to fetch dominant color data
    pub fn fetch_dominant_colors(&mut self) -> &mut Self {
        for region in self.regions.iter_mut() {
            self.pool.scoped(|scoped| {
                scoped.execute(|| {
                    region.fetch_dominant_colors();
                });
            });
        }

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

        let mut butterfly_num: usize = 0;
        let mut pdf_num: usize = 0;

        // Not ideal in terms of memory usage
        for region in self.regions.iter() {
            let mut region_butterflies = region
                .butterflies
                .clone()
                .into_iter()
                .map(|(_k, v)| v)
                .collect::<Vec<Butterfly>>();
            pdf_num += region.pdfs.len();
            butterfly_num += region_butterflies.len();
            butterflies.append(&mut region_butterflies);
        }

        let butterfly_json = ButterflyJSON::new(&butterflies, butterfly_num, pdf_num);

        let json_file = File::create(dir_path.join(JSON_FILE_NAME))?;
        serde_json::to_writer_pretty(json_file, &butterfly_json)?;

        Ok(())
    }
}

use serde::{Deserialize, Serialize};

///Struct used to export data as JSON
#[derive(Deserialize, Serialize, Debug, PartialEq, PartialOrd)]
pub struct ButterflyJSON {
    butterflies: Vec<Butterfly>,
    butterfly_num: usize,
    pdf_num: usize,
    created_at: SystemTime,
}

impl ButterflyJSON {
    fn new(butterflies: &Vec<Butterfly>, butterfly_num: usize, pdf_num: usize) -> Self {
        let created_at = SystemTime::now();

        ButterflyJSON {
            butterflies: butterflies.clone(),
            butterfly_num,
            pdf_num,
            created_at,
        }
    }
}
