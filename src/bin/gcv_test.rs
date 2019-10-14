use butterfly::cloud_vision;
use reqwest;

fn main() {
    let url =
        reqwest::Url::parse("http://biokite.com/worldbutterfly/pa-sp/saty/hikage-com.jpg")
            .unwrap();
    let cols = cloud_vision::get_dominant_colors(&url).unwrap();

    println!("{:#?}", cols);
}
