use butterfly::cloud_vision;
use reqwest;

fn main() {
    let url = reqwest::Url::parse(
        "http://www.ibukiyama-driveway.jp/images/flower/flower_20191006175111_02.jpg",
    )
    .unwrap();
    let cols = cloud_vision::get_dominant_colors(&url).unwrap();

    println!("{:#?}", cols);
}