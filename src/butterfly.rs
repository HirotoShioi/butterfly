use super::cloud_vision::Color;
use super::csv_data::CSVData;
use kanaria::UCSStr;
use serde::{Deserialize, Serialize};

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

    /// Add datas from CSV file
    pub fn add_csv_data(&mut self, csv_data: &CSVData) {
        self.distribution = csv_data.distribution.to_owned();
        self.open_length = csv_data.open_length;
        self.diet = csv_data.diet.to_owned();
        self.remarks = csv_data.remarks.to_owned();
    }
}
