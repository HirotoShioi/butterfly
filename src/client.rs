use std::fs::{create_dir_all, remove_dir_all, File};
use std::io;
use std::path::Path;

use super::constants::*;
use super::webpage_parser::WebpageParser;
use super::butterfly_region::{Butterfly, ButterflyRegion};


pub struct Client {
    targets: Vec<WebpageParser>,
}

impl Client {
    pub fn new(targets: Vec<WebpageParser>) -> Client {
        Client {
            targets
        }
    }

    pub fn fetch_datas(&mut self) -> ButterflyRegions {
        let mut regions = Vec::new();

        for target in self.targets.iter_mut() {
            //Proper exception handling
            let region = target.fetch_data().unwrap();
            regions.push(region);
        }

        ButterflyRegions {
            regions
        }
    }
}

pub struct ButterflyRegions {
    regions: Vec<ButterflyRegion>,
}

impl ButterflyRegions {

    pub fn fetch_images(&mut self) -> &mut Self { 
        for region in self.regions.iter_mut() {
            region.fetch_images();
        }

        self
    }

    pub fn fetch_pdfs(&mut self) -> &mut Self {
        for region in self.regions.iter_mut() {
            region.fetch_pdfs();
        }

        self
    }

    pub fn fetch_dominant_colors(&mut self) -> &mut Self {
        for region in self.regions.iter_mut() {
            region.fetch_dominant_colors();
        }

        self
    }

    pub fn store_json(&mut self) -> Result<(), io::Error> {
        let mut butterflies = Vec::new();

        let dir_path = Path::new(ASSET_DIRECTORY);

        if create_dir_all(&dir_path).is_err() {
            remove_dir_all(&dir_path)?;
            create_dir_all(&dir_path)?;
        };

        for region in self.regions.iter() {
            let region_butterflies = region.butterflies.values().collect::<Vec<&Butterfly>>();
            butterflies.push(region_butterflies);
        }

        let json_file = File::create(dir_path.join(JSON_FILE_NAME))?;
        serde_json::to_writer_pretty(json_file, &butterflies)?;

        Ok(())
    }
}
