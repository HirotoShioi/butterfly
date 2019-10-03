use butterfly::ButterflyRegion;

fn main() {
    let mut regions = vec![
        ButterflyRegion::new(
            "old_north",
            "http://biokite.com/worldbutterfly/butterfly-PArc.htm#PAall",
        ),
        ButterflyRegion::new(
            "new_north",
            "http://biokite.com/worldbutterfly/butterfly-NArc.htm#NAsa",
        ),
        ButterflyRegion::new(
            "new_tropical",
            "http://biokite.com/worldbutterfly/butterfly-NTro.htm#NTmap",
        ),
        ButterflyRegion::new(
            "india_australia",
            "http://biokite.com/worldbutterfly/butterfly-IOrs.htm#IOmap",
        ),
        ButterflyRegion::new(
            "tropical_africa",
            "http://biokite.com/worldbutterfly/butterfly-TAfr.htm#TAmaps",
        ),
    ];

    for region in regions.iter_mut() {
        let url = region.url.to_owned();
        region
            .start()
            .unwrap_or_else(|_| panic!("Failed to extract data from: {}", url));
        // region.fetch_images();
    }
}
