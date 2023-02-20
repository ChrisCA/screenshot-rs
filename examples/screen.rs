use std::time::Instant;

use image::RgbaImage;
use screenshot::get_screenshot;

fn main() {
    let instant = Instant::now();
    println!("Started after: {}", instant.elapsed().as_millis()); // 50 - 60 ms
    let s = get_screenshot().unwrap();
    println!("Got screenshot after: {}", instant.elapsed().as_millis()); // 50 - 60 ms

    let img2 =
        RgbaImage::from_raw(s.width as u32, s.height as u32, s.data_r_and_b_switched).unwrap();

    // 10 - 15 ms
    println!(
        "Transformed image to other type: {}",
        instant.elapsed().as_millis()
    );

    img2.save_with_format("test_vec.bmp", image::ImageFormat::Bmp)
        .unwrap();
    println!(
        "Saved second image after: {}",
        instant.elapsed().as_millis()
    ); // 25 ms

    img2.save_with_format("test2_vec.png", image::ImageFormat::Png)
        .unwrap();
    println!("Saved first image after: {}", instant.elapsed().as_millis()); // 45 - 55 ms

    image::save_buffer(
        "test.png",
        &s.data,
        s.width as u32,
        s.height as u32,
        image::ColorType::Rgba8, // RGBA(8),
    )
    .unwrap();
    println!("Saved third image after: {}", instant.elapsed().as_millis()); // 45 - 55 ms
}
