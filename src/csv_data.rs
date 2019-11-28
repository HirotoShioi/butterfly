use csv::StringRecord;
use kana;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::constants::*;
use super::errors::ButterflyError::{self, *};

/// CSV data extracted from `butterfly.csv`
#[derive(Debug, PartialEq, PartialOrd, Clone, Serialize, Deserialize)]
pub struct CSVData {
    pub distribution: String,
    pub open_length: u32,
    pub diet: Option<String>,
    pub remarks: Option<String>,
}

#[derive(Eq, Debug, PartialEq, Hash, Clone)]
pub struct JPName(pub String);

#[derive(Eq, Debug, PartialEq, Hash, Clone)]
pub struct EngName(pub String);

impl CSVData {
    /// Create an new instance of `CSVData`
    pub fn new(vec: StringRecord) -> Option<((JPName, EngName), CSVData)> {
        let eng_name = vec.get(0)?;
        let jp_name = vec.get(1)?;
        let open_length = vec.get(3).and_then(|num| {
            let parsed: Option<u32> = num.parse().ok();
            parsed
        })?;

        let distribution = vec.get(4).map(|v| normalize(v))?;

        let diet = vec.get(5).and_then(|d| {
            if d.is_empty() {
                None
            } else {
                Some(normalize(d))
            }
        });
        let remarks = vec.get(6).and_then(|r| {
            if r.is_empty() {
                None
            } else {
                Some(normalize(r))
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

/// Fetch `CSVData` from CSV file in which the filepath is `CSV_FILE_PATH`
pub fn fetch_csv_data() -> Result<HashMap<(JPName, EngName), CSVData>, ButterflyError> {
    let mut csv_data_map = HashMap::new();
    // Read file
    let mut cvs_file_content = csv::Reader::from_path(CSV_FILE_PATH)
        .map_err(|_e| FileNotFound(CSV_FILE_PATH.to_owned()))?;

    for record in cvs_file_content.records() {
        let record =
            record.or_else(|_err| Err(FailedToParseCSVRecord(CSV_FILE_PATH.to_string())))?;
        if let Some((key, csv_data)) = CSVData::new(record) {
            csv_data_map.insert(key, csv_data);
        } else {
            return Err(FailedToParseCSVRecord(CSV_FILE_PATH.to_string()));
        };
    }

    Ok(csv_data_map)
}

pub fn normalize(text: &str) -> String {
    let result = kana::wide2ascii(text);
    let result = kana::nowidespace(&result);
    kana::half2kana(&result)
}
