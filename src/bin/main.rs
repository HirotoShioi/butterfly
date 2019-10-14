use butterfly::Client;

fn main() {
    let mut regions = vec![
        Client::new(
            "old_north",
            "旧北区",
            "http://biokite.com/worldbutterfly/butterfly-PArc.htm#PAall",
        ),
        Client::new(
            "new_north",
            "新北区",
            "http://biokite.com/worldbutterfly/butterfly-NArc.htm#NAsa",
        ),
        Client::new(
            "new_tropical",
            "新熱帯区",
            "http://biokite.com/worldbutterfly/butterfly-NTro.htm#NTmap",
        ),
        Client::new(
            "india_australia",
            "インド・オーストラリア区",
            "http://biokite.com/worldbutterfly/butterfly-IOrs.htm#IOmap",
        ),
        Client::new(
            "tropical_africa",
            "熱帯アフリカ区",
            "http://biokite.com/worldbutterfly/butterfly-TAfr.htm#TAmaps",
        ),
    ];

    for region in regions.iter_mut() {
        let url = region.url.to_owned();
        println!("{}", &region.region);
        region
            .start()
            .unwrap_or_else(|_| panic!("Failed to extract data from: {}", url));
        println!("{:#?}", region);
    }

}
