use butterfly::Client;

fn main() {
    let mut clients = vec![
        Client::new(
            "old_north",
            "旧北区",
            "http://biokite.com/worldbutterfly/butterfly-PArc.htm#PAall",
        ),
        // Client::new(
        //     "new_north",
        //     "新北区",
        //     "http://biokite.com/worldbutterfly/butterfly-NArc.htm#NAsa",
        // ),
        // Client::new(
        //     "new_tropical",
        //     "新熱帯区",
        //     "http://biokite.com/worldbutterfly/butterfly-NTro.htm#NTmap",
        // ),
        // Client::new(
        //     "india_australia",
        //     "インド・オーストラリア区",
        //     "http://biokite.com/worldbutterfly/butterfly-IOrs.htm#IOmap",
        // ),
        // Client::new(
        //     "tropical_africa",
        //     "熱帯アフリカ区",
        //     "http://biokite.com/worldbutterfly/butterfly-TAfr.htm#TAmaps",
        // ),
    ];

    for client in clients.iter_mut() {
        let url = client.url.to_owned();
        println!("{}", &client.region);
        let mut region = client
            .fetch_data()
            .unwrap_or_else(|_| panic!("Failed to extract data from: {}", url));

        region.fetch_images().fetch_dominant_colors();
        println!("{:#?}", region);
    }
}
