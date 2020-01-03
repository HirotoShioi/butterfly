use butterfly_extractor::{Client, WebpageParser};
use env_logger::Builder;
use log::LevelFilter;
extern crate clap;

use clap::{App, Arg};
use log::info;

fn main() {
    Builder::from_default_env()
        .filter_level(LevelFilter::Info)
        .default_format_module_path(false)
        .default_format_timestamp(false)
        .init();

    let mut client = Client::new(vec![
        WebpageParser::new(
            "old_north",
            "旧北区",
            "http://biokite.com/worldbutterfly/butterfly-PArc.htm#PAall",
        ),
        WebpageParser::new(
            "new_north",
            "新北区",
            "http://biokite.com/worldbutterfly/butterfly-NArc.htm#NAsa",
        ),
        WebpageParser::new(
            "new_tropical",
            "新熱帯区",
            "http://biokite.com/worldbutterfly/butterfly-NTro.htm#NTmap",
        ),
        WebpageParser::new(
            "india_australia",
            "インド・オーストラリア区",
            "http://biokite.com/worldbutterfly/butterfly-IOrs.htm#IOmap",
        ),
        WebpageParser::new(
            "tropical_africa",
            "熱帯アフリカ区",
            "http://biokite.com/worldbutterfly/butterfly-TAfr.htm#TAmaps",
        ),
    ]);

    // CLI here
    let matches = App::new("My Super Program")
        .version("1.0")
        .author("Hiroto Shioi <shioihi@me.com>")
        .about("Does awesome things")
        .arg(
            Arg::with_name("image")
                .short("i")
                .long("image")
                .help("Download images")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("pdf")
                .short("p")
                .long("pdf")
                .help("Download pdf files")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("dominant_colors")
                .short("d")
                .long("dominant")
                .help("Use google cloud vision api to obtain dominant color data")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("json")
                .short("j")
                .long("json")
                .help("Store data into json file")
                .takes_value(true),
        )
        .get_matches();

    let mut butterfly_data = client.collect_datas().unwrap();
    butterfly_data.fetch_csv_info();

    if matches.is_present("image") {
        butterfly_data.fetch_images();
    }

    if matches.is_present("pdf") {
        butterfly_data.fetch_pdfs();
    }

    if matches.is_present("dominant_colors") {
        butterfly_data.fetch_dominant_colors();
    }

    if let Some(file_path) = matches.value_of("json") {
        butterfly_data.store_json(file_path).unwrap();
    }

    info!("{:#?}", butterfly_data);
}
