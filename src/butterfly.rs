use csv::StringRecord;
use kanaria::UCSStr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::cloud_vision::Color;
use super::constants::*;
use super::errors::ButterflyError::{self, *};

#[derive(Eq, Debug, PartialEq, Hash, Clone)]
pub struct JPName(pub String);

#[derive(Eq, Debug, PartialEq, Hash, Clone)]
pub struct EngName(pub String);

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
    pub distribution: String,
    pub open_length: u32,
    pub diet: Option<String>,
    pub remarks: Option<String>,
    /// List of dominant colors
    pub dominant_colors: Vec<Color>,
    pub dir_name: String,
    pub url: String,
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
        dirname: &str,
        url: &str,
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
            distribution: String::new(),
            dir_name: String::from(dirname),
            url: String::from(url),
            open_length: 0,
            diet: None,
            remarks: None,
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

    pub fn add_additional_data(&mut self, csv_data: &CSVData) {
        self.distribution = csv_data.distribution.to_owned();
        self.open_length = csv_data.open_length;
        self.diet = csv_data.diet.to_owned();
        self.remarks = csv_data.remarks.to_owned();
    }
}

// Place it in seperate module

#[derive(Debug, PartialEq, PartialOrd, Clone, Serialize, Deserialize)]
pub struct CSVData {
    distribution: String,
    open_length: u32,
    diet: Option<String>,
    remarks: Option<String>,
}

impl CSVData {
    pub fn new(vec: StringRecord) -> Option<((JPName, EngName), CSVData)> {
        let eng_name = vec.get(0)?;
        let jp_name = vec.get(1)?;
        let open_length = vec.get(3).and_then(|num| {
            let parsed: Option<u32> = num.parse().ok();
            parsed
        })?;

        let distribution = vec.get(4).map(|v| v.to_owned())?;

        let diet = vec.get(5).and_then(|d| {
            if d.is_empty() {
                None
            } else {
                Some(d.to_owned())
            }
        });
        let remarks = vec.get(6).and_then(|r| {
            if r.is_empty() {
                None
            } else {
                Some(r.to_owned())
            }
        });

        let csv_data = CSVData {
            distribution,
            open_length,
            diet,
            remarks,
        };

        Some((
            (JPName(jp_name.to_owned()), EngName(eng_name.to_owned())),
            csv_data,
        ))
    }
}

pub fn fetch_csv_data() -> Result<HashMap<(JPName, EngName), CSVData>, ButterflyError> {
    let mut csv_data_map = HashMap::new();
    // Read file
    let mut cvs_file_content = csv::Reader::from_path(CSV_FILE_PATH).expect("CSV file not found");

    for record in cvs_file_content.records() {
        let record = record.or_else(|_err| Err(FailedToParseCSVRecord))?;
        if let Some((key, csv_data)) = CSVData::new(record) {
            csv_data_map.insert(key, csv_data);
        } else {
            return Err(FailedToParseCSVRecord);
        };
    }

    Ok(csv_data_map)
}
