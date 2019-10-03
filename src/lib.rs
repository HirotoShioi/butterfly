extern crate kanaria;
extern crate reqwest;
extern crate scraper;

use kanaria::UCSStr;
use reqwest::StatusCode;
use scraper::{ElementRef, Html, Selector};
use std::collections::HashMap;
use std::fmt;
use std::fs::{create_dir_all, remove_dir_all, File};
use std::io;

const WEBSITE_CHARSET: &str = "Shift-JIS";
const HOST_URL: &str = "http://biokite.com/worldbutterfly/";
const DIRECTORY_NAME: &str = "images/";

type Id = usize;

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Butterfly {
    /// Url of an image
    img_src: String,
    img_path: Option<String>,
    /// Japanese name
    jp_name: String,
    /// English name
    eng_name: String,
    /// Background color in 6 digit Hex
    bgcolor: String,
}

impl Butterfly {
    ///Creates an instance of `Butterfly`
    ///
    /// `jp_name` and `eng_name` is empty due to the structure of the website
    fn new(img_src: &str, bgcolor: &str) -> Butterfly {
        let jp_name = String::new();
        let eng_name = String::new();
        Butterfly {
            img_src: img_src.to_string(),
            img_path: None,
            jp_name,
            eng_name,
            bgcolor: bgcolor.to_string(),
        }
    }

    ///Add both English and Japanese name to given `Butterfly`
    fn add_names(&mut self, jp_name: &str, eng_name: &str) -> bool {
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

#[derive(Debug)]
pub enum RegionError {
    ImageSourceNotFound,
    TextNotFound,
    InvalidIndexButterflyNotFound,
    FailedToFetchHTML,
    ImageNotFound,
    ImageNameUnknown,
}

impl std::error::Error for RegionError {}

impl fmt::Display for RegionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        let error_message = match self {
            RegionError::ImageSourceNotFound => "Image source not found",
            RegionError::FailedToFetchHTML => "Failed to fetch html",
            RegionError::InvalidIndexButterflyNotFound => "Index of given butterfly does not exist",
            RegionError::TextNotFound => "Text description of a butterfly could not be extracted",
            RegionError::ImageNotFound => "Image could not be fetched",
            RegionError::ImageNameUnknown => "Image name unknown",
        };
        write!(f, "{}", error_message)
    }
}

/// Collections of butterflies categorized by its regions
#[derive(Debug)]
pub struct ButterflyRegion {
    /// Name of the region
    pub name: String,
    /// Url of region page
    pub url: String,
    /// Collections of butterflies
    butterflies: HashMap<Id, Butterfly>,
}

impl ButterflyRegion {
    /// Create an instance of `ButterflyRegion`
    pub fn new(name: &str, url: &str) -> ButterflyRegion {
        let butterflies = HashMap::new();
        ButterflyRegion {
            name: name.to_string(),
            url: url.to_string(),
            butterflies,
        }
    }

    /// Extract informations of butterflies from `url`
    pub fn start(&mut self) -> Result<&mut Self, RegionError> {
        let body = request_html(&self.url).map_err(|_e| RegionError::FailedToFetchHTML)?;
        self.parse_page(&body)?;
        Ok(self)
    }

    /// Insert new `Butterfly` to `butterflies`
    fn insert_butterfly(&mut self, img_src: &str, color: &str) -> Option<&mut ButterflyRegion> {
        let id = self.butterflies.len();
        match self.butterflies.insert(id, Butterfly::new(img_src, color)) {
            Some(_old_val) => None,
            None => Some(self),
        }
    }

    /// Lookup `Butterfly` with given `id`, and update its name
    fn add_names(&mut self, jp_name: &str, eng_name: &str, id: usize) -> bool {
        match self.butterflies.get_mut(&id) {
            Some(butterfly) => {
                butterfly.add_names(jp_name, eng_name);
                true
            }
            None => false,
        }
    }

    // Return Result
    ///Parse given html and extract information from it
    fn parse_page(&mut self, html: &str) -> Result<&mut ButterflyRegion, RegionError> {
        let fragment = Html::parse_document(html);

        // Selectors we would use for parsing
        let table_selector = Selector::parse("table").unwrap();
        let tbody_selector = Selector::parse("tbody").unwrap();
        let tr_selector = Selector::parse("tr").unwrap();
        let td_selector = Selector::parse("td").unwrap();
        let img_selector = Selector::parse("img").unwrap();

        let mut name_id = 0;

        for table in fragment.select(&table_selector) {
            for tbody in table.select(&tbody_selector) {
                for tr in tbody.select(&tr_selector) {
                    if !is_title_section(&tr) {
                        for td in tr.select(&td_selector) {
                            // If a cell has img element, then extract img source
                            // as well as background color
                            if let Some(img) = td.select(&img_selector).next() {
                                if let Some(src) = img.value().attr("src") {
                                    let mut c = "#ffffff";
                                    if let Some(color) = td.value().attr("bgcolor") {
                                        c = color
                                    }
                                    self.insert_butterfly(src, c);
                                } else {
                                    //throw error
                                    return Err(RegionError::ImageSourceNotFound);
                                }
                            // If a cell does not have a img source, then extract
                            // names from it
                            } else {
                                // Ignore empty cell
                                if !is_empty_text(td.clone().text().collect()) {
                                    if let Some((jp_name, eng_name)) = get_jp_en_name(td) {
                                        if self.add_names(&jp_name, &eng_name, name_id) {
                                            name_id += 1;
                                        } else {
                                            return Err(RegionError::InvalidIndexButterflyNotFound);
                                        };
                                    } else {
                                        return Err(RegionError::TextNotFound);
                                    };
                                }
                            };
                        }
                    }
                }
            }
        }

        Ok(self)
    }

    ///Fetch images
    pub fn fetch_images(&mut self) {
        let dir_path = [DIRECTORY_NAME, "/", &self.name].concat();

        if create_dir_all(&dir_path).is_err() {
            remove_dir_all(&dir_path).unwrap();
            create_dir_all(&dir_path).unwrap();
        };

        for (_key, butterfly) in self.butterflies.iter_mut() {
            if let Ok(img_path) = get_image(&self.name, &butterfly.img_src) {
                butterfly.img_path.replace(img_path);
            } else {
                println!("Image not found: {}", &butterfly.jp_name);
            };
        }
    }
}

///Fetch content of given `url`
pub fn request_html(url: &str) -> Result<String, reqwest::Error> {
    let mut req = reqwest::get(url)?;
    req.text_with_charset(WEBSITE_CHARSET)
}

///Check if given tr set are title cells by checking its colspan attributes
fn is_title_section(element: &ElementRef) -> bool {
    let mut is_title = false;
    let td_selector = Selector::parse("td").unwrap();
    if let Some(td) = element.select(&td_selector).next() {
        for attr in td.value().attrs() {
            if attr.0 == "colspan" {
                is_title = true;
            }
        }
    };

    is_title
}

///Checks if given `String` is consisted by whitespaces
fn is_empty_text(str: String) -> bool {
    str.trim_start().is_empty()
}

///Extract both Japanese and English name from given `ElementRef`
fn get_jp_en_name(td: ElementRef) -> Option<(String, String)> {
    let td = td.text().collect::<String>();

    let mut names = vec![];
    for line in td.lines() {
        names.push(line);
    }

    let jp_name;
    let eng_name;

    //Handling exceptions
    if names == vec!["ヒメアカタテハCynthia_cardui"] {
        jp_name = Some("ヒメアカタテハ");
        eng_name = Some("Cynthia_cardui");
    } else if names == vec!["ツマムラサキマダラ♀Euploea_mulcibe"] {
        jp_name = Some("ツマムラサキマダラ♀");
        eng_name = Some("Euploea_mulcibe");
    } else if names.get(0).cloned() == Some("ミイロタイマイ") {
        jp_name = Some("ミイロタイマイ");
        eng_name = Some("Graphium_weiskei");
    } else {
        jp_name = names.get(0).cloned();
        eng_name = names.get(1).cloned();
    }

    match (jp_name, eng_name) {
        (Some(jp), Some(eng)) => {
            let eng = eng.trim_end().trim_start().to_string();
            Some((jp.to_string(), eng))
        }
        _ => None,
    }
}

///Fetch image from biokite.com and store them on a directory
///
/// Will return `Error` type if,
///
/// 1. Image could not be fetched (either connnection issue or status code other than `Ok`)
/// 2. Image name is unknown (very unlikely to happen)
/// 3. File could not be created
/// 4. Writing to file failed
pub fn get_image(subdir: &str, url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let img_url = [HOST_URL, url].concat();
    let mut response = reqwest::get(&img_url)?;

    if response.status() != StatusCode::OK {
        return Err(Box::new(RegionError::ImageNotFound));
    }

    let fname = response
        .url()
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) });

    match fname {
        None => Err(Box::new(RegionError::ImageNameUnknown)),
        Some(name) => {
            let file_path = [DIRECTORY_NAME, subdir, "/", name].concat();
            //Convert to half-width since some of the are mixed with full and half width
            let file_path = UCSStr::from_str(&file_path).narrow().to_string();
            let mut out = File::create(&file_path)?;
            io::copy(&mut response, &mut out)?;
            Ok(file_path)
        }
    }
}
