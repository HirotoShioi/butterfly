use scoped_threadpool;
use std::fs::{create_dir_all, remove_dir_all, File};
use std::io;
use std::path::Path;

use super::butterfly_region::{Butterfly, ButterflyRegion};
use super::constants::*;
use super::webpage_parser::WebpageParser;

pub struct Client {
    targets: Vec<WebpageParser>,
    pool: scoped_threadpool::Pool,
}

impl Client {
    pub fn new(targets: Vec<WebpageParser>) -> Client {
        let pool = scoped_threadpool::Pool::new(CLIENT_POOL_NUM);
        Client { targets, pool }
    }

    pub fn fetch_datas(&mut self) -> ButterflyRegions {
        let mut regions = Vec::new();

        for target in self.targets.iter_mut() {
            //Proper exception handling
            self.pool.scoped(|scoped| {
                scoped.execute(|| {
                    let region = target.fetch_data().unwrap();
                    regions.push(region);
                });
            });
        }

        let pool = scoped_threadpool::Pool::new(CLIENT_POOL_NUM);

        ButterflyRegions { regions, pool }
    }
}

pub struct ButterflyRegions {
    regions: Vec<ButterflyRegion>,
    pool: scoped_threadpool::Pool,
}

impl ButterflyRegions {
    pub fn fetch_images(&mut self) -> &mut Self {
        for region in self.regions.iter_mut() {
            self.pool.scoped(|scoped| {
                scoped.execute(|| {
                    region.fetch_images();
                });
            });
        }

        self
    }

    pub fn fetch_pdfs(&mut self) -> &mut Self {
        for region in self.regions.iter_mut() {
            self.pool.scoped(|scoped| {
                scoped.execute(|| {
                    region.fetch_pdfs();
                });
            });
        }

        self
    }

    pub fn fetch_dominant_colors(&mut self) -> &mut Self {
        for region in self.regions.iter_mut() {
            self.pool.scoped(|scoped| {
                scoped.execute(|| {
                    region.fetch_dominant_colors();
                });
            });
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
            let mut region_butterflies = region.butterflies.values().collect::<Vec<&Butterfly>>();
            butterflies.append(&mut region_butterflies);
        }

        let json_file = File::create(dir_path.join(JSON_FILE_NAME))?;
        serde_json::to_writer_pretty(json_file, &butterflies)?;

        Ok(())
    }
}
