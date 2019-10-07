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
use std::path::{Path, PathBuf};

const WEBSITE_CHARSET: &str = "Shift-JIS";
const HOST_URL: &str = "http://biokite.com/worldbutterfly/";
const ASSET_DIRECTORY: &str = "./assets";
const IMAGE_DIRECTORY: &str = "images";

type Id = usize;

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Butterfly {
    /// Category
    category: String,
    /// Url of an image
    img_src: String,
    /// Path to image
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
    fn new(img_src: &str, bgcolor: &str, category: &str) -> Butterfly {
        let jp_name = String::new();
        let eng_name = String::new();
        Butterfly {
            category: category.to_string(),
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
pub enum ButterflyRegionError {
    ImageSourceNotFound,
    TextNotFound,
    InvalidIndexButterflyNotFound,
    FailedToFetchHTML,
    ImageNotFound,
    ImageNameUnknown,
}

impl std::error::Error for ButterflyRegionError {}

impl fmt::Display for ButterflyRegionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        let error_message = match self {
            ButterflyRegionError::ImageSourceNotFound => "Image source not found",
            ButterflyRegionError::FailedToFetchHTML => "Failed to fetch html",
            ButterflyRegionError::InvalidIndexButterflyNotFound => "Index of given butterfly does not exist",
            ButterflyRegionError::TextNotFound => "Text description of a butterfly could not be extracted",
            ButterflyRegionError::ImageNotFound => "Image could not be fetched",
            ButterflyRegionError::ImageNameUnknown => "Image name unknown",
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
    pub fn start(&mut self) -> Result<&mut Self, ButterflyRegionError> {
        let body = request_html(&self.url).map_err(|_e| ButterflyRegionError::FailedToFetchHTML)?;
        self.parse_page(&body)?;
        Ok(self)
    }

    /// Insert new `Butterfly` to `butterflies`
    fn insert_butterfly(
        &mut self,
        img_src: &str,
        color: &str,
        category: &str,
    ) -> Option<&mut ButterflyRegion> {
        let id = self.butterflies.len();
        match self
            .butterflies
            .insert(id, Butterfly::new(img_src, color, category))
        {
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
    fn parse_page(&mut self, html: &str) -> Result<&mut ButterflyRegion, ButterflyRegionError> {
        let fragment = Html::parse_document(html);

        // Selectors we would use for parsing
        let table_selector = Selector::parse("table").unwrap();
        let tbody_selector = Selector::parse("tbody").unwrap();
        let tr_selector = Selector::parse("tr").unwrap();
        let td_selector = Selector::parse("td").unwrap();
        let img_selector = Selector::parse("img").unwrap();

        let mut name_id = 0;
        let mut color_category_map: HashMap<String, String> = HashMap::new();
        let mut table_color = "#ffffff";

        for table in fragment.select(&table_selector) {
            if let Some(color) = table.value().attr("bgcolor") {
                table_color = color;
            };
            for tbody in table.select(&tbody_selector) {
                for tr in tbody.select(&tr_selector) {
                    if !is_category_section(&tr) {
                        for td in tr.select(&td_selector) {
                            // If a cell has img element, then extract img source
                            // as well as background color
                            if let Some(img) = td.select(&img_selector).next() {
                                if let Some(src) = img.value().attr("src") {
                                    let (color, category) = extract_color_category(
                                        src,
                                        table_color,
                                        td,
                                        &color_category_map,
                                    );
                                    self.insert_butterfly(src, color, &category);
                                } else {
                                    //throw error
                                    return Err(ButterflyRegionError::ImageSourceNotFound);
                                }
                            // If a cell does not have a img source, then extract
                            // names from it
                            } else {
                                // Ignore empty cell
                                if !is_empty_text(&(td.clone().text().collect::<String>())) {
                                    if let Some((jp_name, eng_name)) = get_jp_en_name(td) {
                                        if self.add_names(&jp_name, &eng_name, name_id) {
                                            name_id += 1;
                                        } else {
                                            return Err(ButterflyRegionError::InvalidIndexButterflyNotFound);
                                        };
                                    } else {
                                        return Err(ButterflyRegionError::TextNotFound);
                                    };
                                }
                            };
                        }
                    } else {
                        //Extract category and its color
                        let vecs = extract_color_category_vec(table_color, &tr);
                        for (color, category) in vecs {
                            color_category_map.insert(color, category);
                        }
                    }
                }
            }
        }

        Ok(self)
    }

    ///Fetch images
    pub fn fetch_images(&mut self) {
        let dir_path = Path::new(ASSET_DIRECTORY)
            .join(IMAGE_DIRECTORY)
            .join(&self.name);

        if create_dir_all(&dir_path).is_err() {
            remove_dir_all(&dir_path).unwrap();
            create_dir_all(&dir_path).unwrap();
        };

        for (_key, butterfly) in self.butterflies.iter_mut() {
            let url = [HOST_URL, &butterfly.img_src].concat();
            if let Ok(img_path) = download_file(&dir_path, &url) {
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

///Check if given tr set are category cells
fn is_category_section(element: &ElementRef) -> bool {
    let mut is_category = false;
    let td_selector = Selector::parse("td").unwrap();
    if let Some(td) = element.select(&td_selector).next() {
        let has_colspan = td.value().attr("colspan").is_some();
        // Table cell with these attributes are not category cell, so ignore them
        let has_bgcolor = td.value().attr("bgcolor") == Some("#ffff66");
        let has_width = td.value().attr("width") == Some("337");

        if has_width && has_bgcolor {
            is_category = false;
        } else if has_colspan {
            is_category = true;
        }
    };

    is_category
}

///Checks if given `String` is consisted by whitespaces
fn is_empty_text(str: &str) -> bool {
    str.trim_start().is_empty()
}

///Extract vectors of color and its category
fn extract_color_category_vec(table_color: &str, element: &ElementRef) -> Vec<(String, String)> {
    let td_selector = Selector::parse("td").unwrap();
    let mut pairs = Vec::new();

    for td in element.select(&td_selector) {
        let category = td.text().find(|txt| txt.contains("科")).unwrap_or("");
        if !is_empty_text(category) {
            if let Some(color) = td.value().attr("bgcolor") {
                pairs.push((color.to_string(), category.to_string()));
            } else {
                pairs.push((table_color.to_string(), category.to_string()));
            }
        }
    }

    pairs
}

///Extract Color and Category string from given `td` cell.
fn extract_color_category<'a>(
    src: &'a str,
    table_color: &'a str,
    td: ElementRef<'a>,
    color_category_map: &'a HashMap<String, String>,
) -> (&'a str, &'a str) {
    let color = if let Some(c) = td.value().attr("bgcolor") {
        c
    } else {
        table_color
    };
    let category = match color_category_map.get(color) {
        Some(t) => t,
        None => {
            if color == "#ffff66" {
                "アゲハチョウ科"
            } else {
                println!("category not found: {}", src);
                ""
            }
        }
    };

    (color, category)
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

///Fetch file from biokite.com and store them on a directory
///
/// Will return `Error` type if,
///
/// 1. Image could not be fetched (either connnection issue or status code other than `Ok`)
/// 2. Image name is unknown (very unlikely to happen)
/// 3. File could not be created
/// 4. Writing to file failed
pub fn download_file(
    directory: &PathBuf,
    url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
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
