use kanaria::UCSStr;
use log::{trace, warn};
use reqwest::{StatusCode, Url};
use scoped_threadpool;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::{create_dir_all, remove_dir_all, File};
use std::io;
use std::path::{Path, PathBuf};

use super::cloud_vision::{get_dominant_colors, Color};
use super::constants::*;
use super::errors::ButterflyError;

/// Buttterfly struct
#[derive(Debug, PartialEq, PartialOrd, Clone, Serialize, Deserialize)]
pub struct Butterfly {
    /// Region
    pub region: String,
    /// Category
    pub category: String,
    /// Url of an image
    pub img_src: String,
    /// Url to pdf
    pub pdf_src: String,
    /// Path to image
    pub img_path: Option<String>,
    /// Path to pdf file
    pub pdf_path: String,
    /// Japanese name
    pub jp_name: String,
    /// English name
    pub eng_name: String,
    /// Background color in 6 digit Hex
    pub bgcolor: String,
    /// List of dominant colors
    pub dominant_colors: Vec<Color>,
}

impl Butterfly {
    /// Creates an instance of `Butterfly`
    ///
    /// Initially, `jp_name` and `eng_name` is empty due to the structure of the website
    pub fn new(
        region: &str,
        img_src: &str,
        pdf_src: &str,
        bgcolor: &str,
        category: &str,
    ) -> Butterfly {
        Butterfly {
            region: String::from(region),
            category: String::from(category),
            img_src: String::from(img_src),
            pdf_src: String::from(pdf_src),
            img_path: None,
            pdf_path: String::new(),
            jp_name: String::new(),
            eng_name: String::new(),
            bgcolor: String::from(bgcolor),
            dominant_colors: Vec::new(),
        }
    }

    ///Add both English and Japanese name to given `Butterfly`
    pub fn add_names(&mut self, jp_name: &str, eng_name: &str) -> bool {
        if self.jp_name.is_empty() {
            let fixed_eng_name = UCSStr::from_str(eng_name).narrow().to_string();
            let fixed_jp_name = UCSStr::from_str(&jp_name)
                .wide()
                .to_string()
                .replace("\u{3000}", "");
            self.jp_name.push_str(&fixed_jp_name);
            self.eng_name.push_str(&fixed_eng_name);
            true
        } else {
            false
        }
    }
}

///Set of butterflyies
#[derive(Debug, Clone)]
pub struct ButterflyRegion {
    /// Directory used to store assets
    pub dir_name: String,
    /// Name of the region
    pub region: String,
    /// Url of region page
    pub url: String,
    /// Collections of butterflies
    pub butterflies: Vec<Butterfly>,
    /// Pdf collection
    pub pdfs: HashSet<String>,
}

impl ButterflyRegion {
    ///Fetch images of butterflies
    pub fn fetch_images(&mut self) -> &mut Self {
        if self.butterflies.is_empty() {
            panic!("Butterfly data has not been extracted!")
        }

        let dir_path = Path::new(ASSET_DIRECTORY)
            .join(&self.dir_name)
            .join(IMAGE_DIRECTORY);

        if create_dir_all(&dir_path).is_err() {
            remove_dir_all(&dir_path).unwrap();
            create_dir_all(&dir_path).unwrap();
        };

        for butterfly in self.butterflies.iter_mut() {
            let url = Url::parse(BUTTERFLY_URL)
                .unwrap()
                .join(&butterfly.img_src)
                .unwrap();
            if let Ok(img_path) = download_file(&dir_path, url) {
                trace!(
                    "Storing image of {} on the path {}",
                    &butterfly.jp_name,
                    &img_path
                );
                butterfly.img_path.replace(img_path);
            } else {
                warn!("Image could not be fetched: {}", &butterfly.jp_name);
            };
        }

        self
    }

    /// Use Google Cloud Vision API to fetch dominant colors
    pub fn fetch_dominant_colors(&mut self) -> &mut Self {
        if self.pdfs.is_empty() {
            panic!("Butterfly data has not been extracted yet!")
        }

        // Use threadpool
        let mut pool = scoped_threadpool::Pool::new(GCV_THEAD_POOL_NUM);

        pool.scoped(|scoped| {
            for butterfly in self.butterflies.iter_mut() {
                let img_url = Url::parse(BUTTERFLY_URL)
                    .unwrap()
                    .join(&butterfly.img_src)
                    .unwrap();
                scoped.execute(move || match get_dominant_colors(&img_url) {
                    Ok(mut colors) => {
                        trace!("Managed to get dominant colors {}", butterfly.jp_name);
                        butterfly.dominant_colors.append(&mut colors);
                    }
                    Err(err) => {
                        warn!("GCV request failed: {}", butterfly.jp_name);
                        warn!("Url: {:#?}", img_url);
                        warn!("Error: {}", err);
                    }
                });
            }
        });

        self
    }

    /// Download PDF files
    pub fn fetch_pdfs(&mut self) -> &mut Self {
        if self.pdfs.is_empty() {
            panic!("Butterfly data has not been extracted yet!")
        }

        let dir_path = Path::new(ASSET_DIRECTORY)
            .join(&self.dir_name)
            .join(PDF_DIRECTORY);

        if create_dir_all(&dir_path).is_err() {
            remove_dir_all(&dir_path).unwrap();
            create_dir_all(&dir_path).unwrap();
        };

        for pdf_url in self.pdfs.iter() {
            let url = Url::parse(BUTTERFLY_URL).unwrap().join(&pdf_url).unwrap();
            match download_file(&dir_path, url) {
                Ok(pdf_path) => {
                    for butterfly in self.butterflies.iter_mut() {
                        if &butterfly.pdf_src == pdf_url {
                            butterfly.pdf_path.push_str(&pdf_path);
                        }
                    }
                    trace!("Stored pdf file on: {}", pdf_path);
                }
                Err(err) => {
                    trace!("Unable to download pdf file: {}", err);
                }
            }
        }

        self
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
fn download_file(directory: &PathBuf, url: Url) -> Result<String, Box<dyn std::error::Error>> {
    let mut response = reqwest::get(url)?;

    if response.status() != StatusCode::OK {
        return Err(Box::new(ButterflyError::ImageNotFound));
    }

    let fname = response
        .url()
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) });

    match fname {
        None => Err(Box::new(ButterflyError::ImageNameUnknown)),
        Some(name) => {
            let file_path = directory.join(name);
            //Convert to half-width since some of the are mixed with full and half width
            //Since we're running on Linux, unwrap() here is fine.
            let file_path = UCSStr::from_str(&file_path.to_str().unwrap())
                .narrow()
                .to_string();
            let mut out = File::create(&file_path)?;
            io::copy(&mut response, &mut out)?;
            Ok(file_path)
        }
    }
}

/// Create a new instance of `ButterflyRegion`
/// Defined as function to prevent it from being exported as public API
pub fn new_region(
    dir_name: &str,
    region: &str,
    url: &str,
    butterflies: &[Butterfly],
    pdfs: &HashSet<String>,
) -> ButterflyRegion {
    ButterflyRegion {
        dir_name: dir_name.to_owned(),
        region: region.to_owned(),
        url: url.to_owned(),
        butterflies: butterflies.to_owned(),
        pdfs: pdfs.to_owned(),
    }
}
