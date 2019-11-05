use log::error;
use scraper::{ElementRef, Html, Selector};
use std::collections::{HashMap, HashSet};

use super::butterfly::{new_region, Butterfly, ButterflyRegion};
use super::errors::ButterflyError;

// Encoding used on the butterfly website
const WEBSITE_CHARSET: &str = "Shift-JIS";

type Id = usize;

/// Collections of butterflies categorized by its regions
#[derive(Debug, Clone)]
pub struct WebpageParser {
    /// Directory name to store assets
    pub dir_name: String,
    /// Name of the region
    pub region: String,
    /// Url of region page
    pub url: String,
    /// Collections of butterflies
    pub butterflies: HashMap<Id, Butterfly>,
    /// Pdf collection
    pub pdfs: HashSet<String>,
}

impl WebpageParser {
    /// Create an instance of `ButterflyRegion`
    pub fn new(dir_name: &str, region: &str, url: &str) -> WebpageParser {
        let butterflies = HashMap::new();
        WebpageParser {
            dir_name: dir_name.to_string(),
            region: region.to_string(),
            url: url.to_string(),
            butterflies,
            pdfs: HashSet::new(),
        }
    }

    /// Extract informations of butterflies from `url`
    pub fn fetch_data(&mut self) -> Result<ButterflyRegion, ButterflyError> {
        let body = request_html(&self.url).map_err(|_e| ButterflyError::FailedToFetchHTML)?;
        self.parse_page(&body)?;

        let butterfly_vector = self
            .butterflies
            .values()
            .map(|v| v.to_owned())
            .collect::<Vec<Butterfly>>();

        new_region(
            &self.dir_name,
            &self.region,
            &self.url,
            &butterfly_vector,
            &self.pdfs,
        )
    }

    /// Insert new `Butterfly` to `butterflies`
    fn insert_butterfly(
        &mut self,
        img_src: &str,
        pdf_src: &str,
        color: &str,
        category: &str,
    ) -> Option<&mut WebpageParser> {
        let id = self.butterflies.len();
        match self.butterflies.insert(
            id,
            Butterfly::new(&self.region, img_src, pdf_src, color, category),
        ) {
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
    fn parse_page(&mut self, html: &str) -> Result<(), ButterflyError> {
        let fragment = Html::parse_document(html);

        // Selectors we would use for parsing
        let table_selector = Selector::parse("table").unwrap();
        let tbody_selector = Selector::parse("tbody").unwrap();
        let tr_selector = Selector::parse("tr").unwrap();
        let td_selector = Selector::parse("td").unwrap();
        let img_selector = Selector::parse("img").unwrap();
        let a_selector = Selector::parse("a").unwrap();

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
                                    //Extract the url to pdf here
                                    let (color, category) = extract_color_category(
                                        src,
                                        table_color,
                                        &td,
                                        &color_category_map,
                                    );
                                    let href = td
                                        .select(&a_selector)
                                        .next()
                                        .unwrap()
                                        .value()
                                        .attr("href")
                                        .unwrap();
                                    self.pdfs.insert(href.to_owned());
                                    self.insert_butterfly(src, href, &color, &category);
                                } else {
                                    //throw error
                                    return Err(ButterflyError::ImageSourceNotFound);
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
                                            return Err(
                                                ButterflyError::InvalidIndexButterflyNotFound,
                                            );
                                        };
                                    } else {
                                        return Err(ButterflyError::TextNotFound);
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

        Ok(())
    }
}

///Fetch content of given `url`
fn request_html(url: &str) -> Result<String, reqwest::Error> {
    let mut req = reqwest::get(url)?;
    req.text_with_charset(WEBSITE_CHARSET)
}

///Check if given tr set are category cells
fn is_category_section(element: &ElementRef) -> bool {
    let mut is_category = false;
    let td_selector = Selector::parse("td").unwrap();
    if let Some(td) = element.select(&td_selector).next() {
        let has_colspan = td.value().attr("colspan").is_some();
        // Table cell with these attributes is not category cell, so ignore them
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
fn extract_color_category(
    src: &str,
    table_color: &str,
    td: &ElementRef,
    color_category_map: &HashMap<String, String>,
) -> (String, String) {
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
                error!("category not found: {}", src);
                ""
            }
        }
    };

    (color.to_string(), category.to_string())
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
