use ai_image::RgbaImage;

fn main() {
    let mut img = RgbaImage::new(1920, 1080);
    let tile = ai_image::open("examples/scaleup/tinycross.png").unwrap();

    ai_image::imageops::tile(&mut img, &tile);
    img.save("tiled_wallpaper.png").unwrap();
}
