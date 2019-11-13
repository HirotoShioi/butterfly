//! # butterfly_extractor
//!
//! This crate attempts to extract data from http://biokite.com/worldbutterfly/butterfly-index.htm
//! Data include images, pdfs, both japanese and English name, as well as
//! background color it is being used.
//!
//! We are also using [Google Cloud Vision API](https://cloud.google.com/vision/?hl=ja)
//! to extract colors from the images
//!
//! ## How to start
//!
//! You'd start using this library by defining an instance of `Client` which takes
//! vector of `WebpageParser`.
//!  
//!
//! ```rust
//!     let mut client = Client::new(vec![
//!        WebpageParser::new(
//!            "old_north",
//!            "旧北区",
//!            "http://biokite.com/worldbutterfly/butterfly-PArc.htm#PAall",
//!        ),
//!        WebpageParser::new(
//!            "new_north",
//!            "新北区",
//!            "http://biokite.com/worldbutterfly/butterfly-NArc.htm#NAsa",
//!        ),
//!        WebpageParser::new(
//!            "new_tropical",
//!            "新熱帯区",
//!         "http://biokite.com/worldbutterfly/butterfly-NTro.htm#NTmap",
//!        ),
//!        WebpageParser::new(
//!            "india_australia",
//!            "インド・オーストラリア区",
//!            "http://biokite.com/worldbutterfly/butterfly-IOrs.htm#IOmap",
//!        ),
//!        WebpageParser::new(
//!            "tropical_africa",
//1            "熱帯アフリカ区",
//!            "http://biokite.com/worldbutterfly/butterfly-TAfr.htm#TAmaps",
//!        ),
//!    ]);
//! ```
//!
//! After that, call `collect_data` to start collect data from the webpage. This
//! will return `ButterflyData` struct which can be used to fetch assets such as
//! jpeg images, pdf files, etc.
//!
//! ```rust
//!     let mut butterfly_data = client.collect_datas();
//!
//!    butterfly_data
//!        .fetch_images()
//!        .fetch_pdfs()
//!        .fetch_dominant_colors()
//!        .unwrap();
//! ```
//!
//! After everything is done, call `store_json` to store the data on json file
//!
//! ```rust
//!        butterfly_data
//!            .fetch_images()
//!            .fetch_pdfs()
//!            .fetch_dominant_colors()
//!            .store_json()
//!            .unwrap();
//! ```

extern crate csv;
extern crate env_logger;
extern crate hex;
extern crate kanaria;
extern crate log;
extern crate rayon;
extern crate reqwest;
extern crate scoped_threadpool;
extern crate scraper;
extern crate serde;
extern crate serde_json;

mod butterfly;
mod butterfly_collector;
mod client;
mod cloud_vision;
mod constants;
mod errors;
mod webpage_parser;

pub use butterfly::Butterfly;
pub use butterfly_collector::ButterflyCollector;
pub use client::Client;
pub use cloud_vision::Color;
pub use errors::ButterflyError;
pub use webpage_parser::WebpageParser;
