use scrape_test::ButterflyRegion;

fn main() {
    let mut regions = vec![
        ButterflyRegion::new(
            "旧北区",
            "http://biokite.com/worldbutterfly/butterfly-PArc.htm#PAall",
        ),
        ButterflyRegion::new(
            "新北区",
            "http://biokite.com/worldbutterfly/butterfly-NArc.htm#NAsa",
        ),
        ButterflyRegion::new(
            "新熱帯区",
            "http://biokite.com/worldbutterfly/butterfly-NTro.htm#NTmap",
        ),
        ButterflyRegion::new(
            "インド・オーストラリア区",
            "http://biokite.com/worldbutterfly/butterfly-IOrs.htm#IOmap",
        ),
        ButterflyRegion::new(
            "熱帯アフリカの蝶",
            "http://biokite.com/worldbutterfly/butterfly-TAfr.htm#TAmaps",
        ),
    ];

    for region in regions.iter_mut() {
        match region.start() {
            Ok(region) => println!("{:#?}", region),
            Err(err) => println!("{:#?}", err),
        }
    }
}
