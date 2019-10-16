use butterfly::{Client, WebpageParser};

fn main() {
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

    let mut regions = client.fetch_datas();
    regions
        .fetch_images()
        .fetch_pdfs()
        .fetch_dominant_colors()
        .store_json();
}
