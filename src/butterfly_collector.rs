//! # Butterfly Collector
//!
//! This module exports `ButterflyCollector` which is used to retrieve assets
//! based upon the data that were previously extraced. You can chain the methods.
//!
//! You should call `store_json` when all the data has been acquired.
use kanaria::UCSStr;
use log::{info, trace, warn};
use reqwest::{StatusCode, Url};
use scoped_threadpool;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::{create_dir_all, remove_dir_all, File};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::butterfly::Butterfly;
use super::cloud_vision::get_dominant_colors;
use super::constants::*;
use super::csv_data::{fetch_csv_data, CSVData, EngName, JPName};
use super::errors::ButterflyError;
use super::webpage_parser::WebpageParseResult;

#[derive(Debug, Clone)]
/// # Butterfly Collector
///
/// This module exports `ButterflyCollector` which is used to retrieve assets
/// based upon the data that were previously extraced. You can chain the methods.
///
/// You should call `store_json` when all the data has been acquired.
pub struct ButterflyCollector {
    /// Collections of butterflies
    pub butterflies: Vec<Butterfly>,
    /// Pdf collection
    pub pdfs: HashSet<(String, String)>,
    /// Datas parsed from csv file
    pub csv_data_map: HashMap<(JPName, EngName), CSVData>,
    /// List of region directories
    pub region_dirs: Vec<String>,
}

impl ButterflyCollector {
    /// Create new instance of `ButterflyCollector` from given `Vec<WebpageParseResult>`
    pub(crate) fn from_parse_result(
        parse_results: Vec<WebpageParseResult>,
    ) -> Result<ButterflyCollector, ButterflyError> {
        let mut butterflies: Vec<Butterfly> = Vec::new();
        let mut pdfs: HashSet<(String, String)> = HashSet::new();
        let mut region_dirs: Vec<String> = Vec::new();
        let csv_data_map = fetch_csv_data()?;

        for result in parse_results.into_iter() {
            let mut butterfly_vector = result
                .butterflies
                .values()
                .map(|v| v.to_owned())
                .collect::<Vec<Butterfly>>();

            butterflies.append(&mut butterfly_vector);

            region_dirs.push(result.dir_name);

            for pdf in result.pdfs.into_iter() {
                pdfs.insert(pdf);
            }
        }

        Ok(ButterflyCollector {
            butterflies,
            pdfs,
            csv_data_map,
            region_dirs,
        })
    }

    /// Fetch data from CSV data map
    pub fn fetch_csv_info(&mut self) -> &mut Self {
        for butterfly in self.butterflies.iter_mut() {
            let jp_name = JPName(butterfly.jp_name.to_owned());
            let eng_name = EngName(butterfly.eng_name.to_owned());

            let csv_data = self.csv_data_map.get(&(jp_name, eng_name));

            match csv_data {
                Some(additional_data) => {
                    butterfly.add_csv_data(additional_data);
                }
                None => {
                    warn!("Data not found: {}", butterfly.jp_name);
                }
            }
        }

        self
    }

    ///Fetch images of butterflies
    pub fn fetch_images(&mut self) -> &mut Self {
        if self.butterflies.is_empty() {
            panic!("Butterfly data has not been extracted!")
        }

        for region_dir_name in self.region_dirs.iter() {
            let dir_path = Path::new(ASSET_DIRECTORY).join(region_dir_name.to_owned());

            let img_path = Path::new(&dir_path).join(IMAGE_DIRECTORY);

            if create_dir_all(&img_path).is_err() {
                remove_dir_all(&img_path).unwrap();
                create_dir_all(&img_path).unwrap();
            };
        }

        let mut pool = scoped_threadpool::Pool::new(100);

        info!("Downloading image files");

        pool.scoped(|scope| {
            self.butterflies.iter_mut().for_each(|butterfly| {
                let dir_path = Path::new(ASSET_DIRECTORY)
                    .join(&butterfly.dir_name)
                    .join(IMAGE_DIRECTORY);

                let url = Url::parse(BUTTERFLY_URL)
                    .unwrap()
                    .join(&butterfly.img_src)
                    .unwrap();

                scope.execute(move || {
                    let file_name = get_file_name(&butterfly.img_src).unwrap();
                    let file_path = dir_path.join(file_name);
                    if let Ok(img_path) = download_file(&file_path, url) {
                        trace!(
                            "Storing image of {} on the path {}",
                            &butterfly.jp_name,
                            &img_path
                        );
                        butterfly.img_path.replace(img_path);
                    } else {
                        warn!("Image could not be fetched: {}", &butterfly.jp_name);
                    };
                });
            });
        });

        info!("Finished downloading all the images!");

        self
    }

    /// Use Google Cloud Vision API to fetch dominant colors
    pub fn fetch_dominant_colors(&mut self) -> &mut Self {
        if self.pdfs.is_empty() {
            panic!("Butterfly data has not been extracted yet!")
        }

        info!("Using Google Cloud Vision to collect image property data");
        // Use threadpool
        let mut pool = scoped_threadpool::Pool::new(GCV_THEAD_POOL_NUM);

        pool.scoped(|scoped| {
            for butterfly in self.butterflies.iter_mut() {
                let img_url = Url::parse(BUTTERFLY_URL)
                    .unwrap()
                    .join(&butterfly.img_src)
                    .unwrap();

                scoped.execute(move || match get_dominant_colors(&img_url) {
                    Ok(colors) => {
                        trace!("Analyzed image data of {}", butterfly.jp_name);
                        butterfly.dominant_colors = colors;
                    }
                    Err(err) => {
                        warn!("GCV request failed: {}", butterfly.jp_name);
                        warn!("Url: {:#?}", img_url);
                        warn!("Error: {}", err);
                    }
                });
            }
        });

        info!("All the images has been analyzed");

        self
    }

    /// Download PDF files
    pub fn fetch_pdfs(&mut self) -> &mut Self {
        if self.pdfs.is_empty() {
            panic!("Butterfly data has not been extracted yet!")
        }

        info!("Downloading pdf files");

        for region_dir_name in self.region_dirs.iter() {
            let dir_path = Path::new(ASSET_DIRECTORY).join(region_dir_name.to_owned());

            let img_path = Path::new(&dir_path).join(PDF_DIRECTORY);

            if create_dir_all(&img_path).is_err() {
                remove_dir_all(&img_path).unwrap();
                create_dir_all(&img_path).unwrap();
            };
        }

        for (pdf_url, dir_name) in self.pdfs.iter() {
            let dir_path = Path::new(ASSET_DIRECTORY)
                .join(&dir_name)
                .join(PDF_DIRECTORY);
            let url = Url::parse(BUTTERFLY_URL).unwrap().join(&pdf_url).unwrap();
            let file_name = get_file_name(pdf_url).unwrap();
            let file_path = dir_path.join(file_name);
            match download_file(&file_path, url) {
                Ok(pdf_path) => {
                    for butterfly in self.butterflies.iter_mut() {
                        if &butterfly.pdf_src == pdf_url {
                            butterfly.pdf_path = pdf_path.to_owned();
                        }
                    }
                    trace!("Stored pdf file on: {}", pdf_path);
                }
                Err(err) => {
                    warn!("Unable to download pdf file: {}", err);
                }
            }
        }

        info!("Finished downloading all the pdf files!");

        self
    }

    /// Store the result as JSON file as `JSON_FILE_NAME`
    pub fn store_json(&mut self) -> Result<(), std::io::Error> {
        let dir_path = Path::new(ASSET_DIRECTORY);

        if create_dir_all(&dir_path).is_err() {
            remove_dir_all(&dir_path)?;
            create_dir_all(&dir_path)?;
        };

        info!(
            "Storing the results to json file on: {}",
            &dir_path.to_str().unwrap()
        );

        let butterfly_num: usize = self.butterflies.len();
        let pdf_num: usize = self.pdfs.len();
        // Remove duplicates
        self.butterflies
            .sort_by(|b1, b2| b1.jp_name.cmp(&b2.jp_name));
        self.butterflies.dedup_by(|b1, b2| b1.jp_name == b2.jp_name);

        let butterfly_json = ButterflyJSON::new(&self.butterflies, butterfly_num, pdf_num);
        let json_file = File::create(dir_path.join(JSON_FILE_NAME))?;
        serde_json::to_writer_pretty(json_file, &butterfly_json)?;

        Ok(())
    }
}

///Fetch file from biokite.com and store them on a directory
///
/// Will return `Error` type if,
///
/// 1. Image could not be fetched (either connnection issue or status code other than `Ok`)
/// 2. Image name is unknown (very unlikely to happen)
/// 3. File could not be created
/// 4. Writing to file failed
fn download_file(file_path: &PathBuf, url: Url) -> anyhow::Result<String> {
    let mut response = reqwest::get(url)?;

    if response.status() != StatusCode::OK {
        anyhow::bail!(ButterflyError::FileNotFound(
            file_path.to_str().unwrap().to_string(),
        ));
    }

    let mut out = File::create(&file_path)?;
    io::copy(&mut response, &mut out)?;
    trace!("Downloaded: {:#?}", file_path);
    Ok(file_path.to_str().unwrap().to_string())
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
    /// Posix time
    pub created_at: u64,
}

impl ButterflyJSON {
    /// Create an instance of `ButterflyJSON`
    fn new(butterflies: &[Butterfly], butterfly_num: usize, pdf_num: usize) -> Self {
        let created_at = now();

        ButterflyJSON {
            butterflies: butterflies.to_owned(),
            butterfly_num,
            pdf_num,
            created_at,
        }
    }

    /// Convert itself into `ButterflyCollector`
    pub(crate) fn into_collector(self) -> Result<ButterflyCollector, ButterflyError> {
        let csv_data_map = fetch_csv_data()?;

        let mut regions: HashSet<String> = HashSet::new();
        let mut butterflies: Vec<Butterfly> = Vec::new();
        let mut pdfs: HashSet<(String, String)> = HashSet::new();

        // self.pdfs
        // .insert((href.to_owned(), self.dir_name.to_owned()));

        for butterfly in self.butterflies.into_iter() {
            regions.insert(butterfly.region.to_owned());
            pdfs.insert((butterfly.pdf_src.to_owned(), butterfly.dir_name.to_owned()));
            butterflies.push(butterfly.to_owned());
        }

        let region_dirs: Vec<String> = regions.into_iter().collect();

        Ok(ButterflyCollector {
            butterflies,
            pdfs,
            csv_data_map,
            region_dirs,
        })
    }
}

/// Return current POSIX time
fn now() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs()
}

/// Extract file name from given `url_path`
fn get_file_name(url_path: &str) -> Option<String> {
    url_path
        .split("/")
        .last()
        .map(|name| UCSStr::from_str(name).narrow().to_string())
}
