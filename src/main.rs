use butterfly::cloud_vision;
use butterfly::Client;
use reqwest;

fn main() {
    let mut regions = vec![
        Client::new(
            "old_north",
            "http://biokite.com/worldbutterfly/butterfly-PArc.htm#PAall",
        ),
        // ButterflyRegion::new(
        //     "new_north",
        //     "http://biokite.com/worldbutterfly/butterfly-NArc.htm#NAsa",
        // ),
        // ButterflyRegion::new(
        //     "new_tropical",
        //     "http://biokite.com/worldbutterfly/butterfly-NTro.htm#NTmap",
        // ),
        // ButterflyRegion::new(
        //     "india_australia",
        //     "http://biokite.com/worldbutterfly/butterfly-IOrs.htm#IOmap",
        // ),
        // ButterflyRegion::new(
        //     "tropical_africa",
        //     "http://biokite.com/worldbutterfly/butterfly-TAfr.htm#TAmaps",
        // ),
    ];

    for region in regions.iter_mut() {
        let url = region.url.to_owned();
        println!("{}", &region.name);
        region
            .start()
            .unwrap_or_else(|_| panic!("Failed to extract data from: {}", url))
            .fetch_images();
        println!("{:#?}", region);
    }

    let url = reqwest::Url::parse(
        "http://www.ibukiyama-driveway.jp/images/flower/flower_20191006175111_02.jpg",
    )
    .unwrap();
    let cols = cloud_vision::get_dominant_colors(&url).unwrap();

    println!("{:#?}", cols);
}
