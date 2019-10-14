use butterfly::cloud_vision;
use reqwest;

fn main() {
    let url = reqwest::Url::parse("https://blog.jetbrains.com/jp/2019/03/22/1797").unwrap();
    let cols = cloud_vision::get_dominant_colors(&url).unwrap();

    println!("{:#?}", cols);
}
