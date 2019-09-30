extern crate kanaria;
extern crate reqwest;
extern crate scraper;

use kanaria::UCSStr;
use scraper::{ElementRef, Html, Selector};
use std::collections::HashMap;

type Id = usize;

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Butterfly {
    img_src: String,
    jp_name: String,
    eng_name: String,
    bgcolor: String,
}

impl Butterfly {
    fn new_src(img_src: &str, bgcolor: &str) -> Butterfly {
        let jp_name = String::new();
        let eng_name = String::new();
        Butterfly {
            img_src: img_src.to_string(),
            jp_name,
            eng_name,
            bgcolor: bgcolor.to_string(),
        }
    }

    fn add_names(&mut self, jp_name: &str, eng_name: &str) -> bool {
        if self.jp_name.len() <= 0 {
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
pub struct Region {
    name: String,
    url: String,
    butterflies: HashMap<Id, Butterfly>,
}

impl Region {
    pub fn new(name: &str, url: &str) -> Region {
        let butterflies = HashMap::new();
        Region {
            name: name.to_string(),
            url: url.to_string(),
            butterflies,
        }
    }

    pub fn start(&mut self) -> Result<&mut Self, reqwest::Error> {
        let body = request_html(&self.url)?;
        self.parse_page(&body);
        Ok(self)
    }

    fn add_src(&mut self, img_src: &str, color: &str) -> Option<&mut Region> {
        let id = self.butterflies.len();
        match self
            .butterflies
            .insert(id, Butterfly::new_src(img_src, color))
        {
            Some(_old_val) => None,
            None => Some(self),
        }
    }

    fn add_names(&mut self, jp_name: &str, eng_name: &str, id: usize) -> Option<&mut Region> {
        match self.butterflies.get_mut(&id) {
            Some(butterfly) => {
                butterfly.add_names(jp_name, eng_name);
                Some(self)
            }
            None => None,
        }
    }

    fn parse_page(&mut self, html: &str) -> &mut Region {
        let fragment = Html::parse_document(html);
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
                            if let Some(img) = td.select(&img_selector).next() {
                                if let Some(src) = img.value().attr("src") {
                                    let mut c = "#ffffff";
                                    if let Some(color) = td.value().attr("bgcolor") {
                                        c = color
                                    }
                                    self.add_src(src, c);
                                }
                            } else {
                                if !is_empty_text(td.clone().text().collect()) {
                                    let (jp_name, eng_name) = get_jp_en_name(td);
                                    self.add_names(&jp_name, &eng_name, name_id);
                                    name_id = name_id + 1;
                                }
                            };
                        }
                    }
                }
            }
        }

        self
    }
}

pub fn request_html(url: &str) -> Result<String, reqwest::Error> {
    let mut req = reqwest::get(url)?;
    let body = req.text_with_charset("Shift-JIS");
    body
}

fn strip_white_spaces(text: &str) -> String {
    let mut text = text.chars().peekable();

    loop {
        if let Some(c) = text.peek() {
            if c.is_ascii_whitespace() {
                text.next();
                continue;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    text.collect()
}

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

fn is_empty_text(str: String) -> bool {
    strip_white_spaces(&str).is_empty()
}

fn get_jp_en_name(td: ElementRef) -> (String, String) {
    let td = td.text().collect::<String>();

    let mut names = vec![];
    for line in td.lines() {
        names.push(line);
    }

    let jp_name;
    let eng_name;

    if names == vec!["ヒメアカタテハCynthia_cardui"] {
        jp_name = "ヒメアカタテハ".to_string();
        eng_name = "Cynthia_cardui".to_string();
    } else if names == vec!["ツマムラサキマダラ♀Euploea_mulcibe"] {
        jp_name = "ツマムラサキマダラ♀".to_string();
        eng_name = "Euploea_mulcibe".to_string();
    } else if names[0] == "ミイロタイマイ".to_string() {
        jp_name = "ミイロタイマイ".to_string();
        eng_name = "Graphium_weiskei".to_string();
    } else {
        jp_name = names[0].to_string();
        eng_name = names[1].to_string();
    }

    let eng_name = strip_white_spaces(&eng_name).trim_end().to_string();

    (jp_name, eng_name)
}
