use color_thief::ColorFormat;
use kanaria::UCSStr;
use reqwest::{StatusCode, Url};
use std::collections::{HashMap, HashSet};
use std::fs::{create_dir_all, remove_dir_all, File};
use std::io;
use std::path::{Path, PathBuf};

use super::errors::ButterflyRegionError;

type Id = usize;

// Url of the website
const BUTTERFLY_URL: &str = "http://biokite.com/worldbutterfly/";
// Directory which stores the downloaded files
const ASSET_DIRECTORY: &str = "./assets";
// Directory which stores the images
const IMAGE_DIRECTORY: &str = "images";
// Directory which store the pdf files
const PDF_DIRECTORY: &str = "pdf";

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Butterfly {
    /// Category
    category: String,
    /// Url of an image
    img_src: String,
    /// Url to pdf
    pdf_src: String,
    /// Path to image
    img_path: Option<String>,
    /// Path to pdf file
    pdf_path: String,
    /// Japanese name
    jp_name: String,
    /// English name
    eng_name: String,
    /// Background color in 6 digit Hex
    bgcolor: String,
    dominant_colors: Vec<String>,
}

impl Butterfly {
    ///Creates an instance of `Butterfly`
    ///
    /// `jp_name` and `eng_name` is empty due to the structure of the website
    pub fn new(img_src: &str, pdf_src: &str, bgcolor: &str, category: &str) -> Butterfly {
        Butterfly {
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

pub struct ButterflyRegion {
    /// Name of the region
    pub name: String,
    /// Url of region page
    pub url: String,
    /// Collections of butterflies
    pub butterflies: HashMap<Id, Butterfly>,
    /// Pdf collection
    pub pdfs: HashSet<String>,
}

impl ButterflyRegion {
    ///Fetch images
    pub fn fetch_images(&mut self) {
        if self.butterflies.is_empty() {
            panic!("Butterfly data has not been extracted!")
        }

        let dir_path = Path::new(ASSET_DIRECTORY)
            .join(&self.name)
            .join(IMAGE_DIRECTORY);

        if create_dir_all(&dir_path).is_err() {
            remove_dir_all(&dir_path).unwrap();
            create_dir_all(&dir_path).unwrap();
        };

        for (_key, butterfly) in self.butterflies.iter_mut() {
            let url = Url::parse(BUTTERFLY_URL)
                .unwrap()
                .join(&butterfly.img_src)
                .unwrap();
            if let Ok(img_path) = download_file(&dir_path, url) {
                butterfly.img_path.replace(img_path);
            } else {
                println!("Image not found: {}", &butterfly.jp_name);
            };
        }
    }

    pub fn fetch_dominant_colors(&mut self) {
        if self.pdfs.is_empty() {
            panic!("Butterfly data has not been extracted yet!")
        }

        for butterfly in self.butterflies.values_mut() {
            let img_url = Url::parse(BUTTERFLY_URL)
                .unwrap()
                .join(&butterfly.img_src)
                .unwrap();
            let mut colors = get_dominant_colors(img_url).unwrap();
            butterfly.dominant_colors.append(&mut colors);
        }
    }

    pub fn fetch_pdfs(&mut self) {
        if self.pdfs.is_empty() {
            panic!("Butterfly data has not been extracted yet!")
        }

        let dir_path = Path::new(ASSET_DIRECTORY)
            .join(&self.name)
            .join(PDF_DIRECTORY);

        if create_dir_all(&dir_path).is_err() {
            remove_dir_all(&dir_path).unwrap();
            create_dir_all(&dir_path).unwrap();
        };

        for pdf_url in self.pdfs.iter() {
            let url = Url::parse(BUTTERFLY_URL).unwrap().join(&pdf_url).unwrap();
            match download_file(&dir_path, url) {
                Ok(pdf_path) => {
                    for butterfly in self.butterflies.values_mut() {
                        if &butterfly.pdf_src == pdf_url {
                            butterfly.pdf_path.push_str(&pdf_path);
                        }
                    }
                    println!("{}", pdf_path);
                }
                Err(err) => {
                    println!("{:?}", err);
                }
            }
        }
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
        return Err(Box::new(ButterflyRegionError::ImageNotFound));
    }

    let fname = response
        .url()
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) });

    match fname {
        None => Err(Box::new(ButterflyRegionError::ImageNameUnknown)),
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

/// Stop using this api!
const IMAGE_QUALITY: u8 = 5;
const COLOR_NUM: u8 = 2;

///Fetch an image, and returns vector of dominant colors
fn get_dominant_colors(url: Url) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut res = reqwest::get(url)?;
    let mut buf: Vec<u8> = vec![];
    res.copy_to(&mut buf)?;

    let colors = color_thief::get_palette(&buf, ColorFormat::Bgr, IMAGE_QUALITY, COLOR_NUM)
        .map_err(|_| ButterflyRegionError::NotImage)?;

    let mut hex_colors: Vec<String> = vec![];

    for color in colors {
        let mut hexcolor = String::from("#");
        let hex = hex::encode(vec![color.r, color.g, color.b]);
        hexcolor.push_str(&hex);
        hex_colors.push(hexcolor);
    }

    Ok(hex_colors)
}
