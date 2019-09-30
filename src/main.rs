use scrape_test::Region;

fn main() {
    let mut butterfly_pages = vec![
        Region::new(
            "旧北区",
            "http://biokite.com/worldbutterfly/butterfly-PArc.htm#PAall",
        ),
        Region::new(
            "新北区",
            "http://biokite.com/worldbutterfly/butterfly-NArc.htm#NAsa",
        ),
        Region::new(
            "新熱帯区",
            "http://biokite.com/worldbutterfly/butterfly-NTro.htm#NTmap",
        ),
        Region::new(
            "インド・オーストラリア区",
            "http://biokite.com/worldbutterfly/butterfly-IOrs.htm#IOmap",
        ),
        Region::new(
            "熱帯アフリカの蝶",
            "http://biokite.com/worldbutterfly/butterfly-TAfr.htm#TAmaps",
        ),
    ];

    for region in butterfly_pages.iter_mut() {
        region.start();
        println!("{:#?}", region);
    }
}
