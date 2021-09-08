use std::time::Instant;

use image::Rgb;
use image::RgbImage;
use screenshot::get_screenshot;

fn main() {
    let instant = Instant::now();
    let s = get_screenshot().unwrap();
    println!("Got screenshot after: {}", instant.elapsed().as_millis()); // 50 - 60 ms

    println!(
        "{} x {} x {} = {} bytes",
        s.height,
        s.width,
        s.pixel_width,
        s.len()
    );

    let origin = s.get_pixel(0, 0);
    println!("(0,0): R: {}, G: {}, B: {}", origin.r, origin.g, origin.b);

    let end_col = s.get_pixel(0, s.width - 1);
    println!(
        "(0,end): R: {}, G: {}, B: {}",
        end_col.r, end_col.g, end_col.b
    );

    let opp = s.get_pixel(s.height - 1, s.width - 1);
    println!("(end,end): R: {}, G: {}, B: {}", opp.r, opp.g, opp.b);

    println!("Got test pixels after: {}", instant.elapsed().as_millis()); // 5 ms

    // WARNING rust-bmp params are (width, height)
    let mut img = RgbImage::new(s.width as u32, s.height as u32);
    for row in 0..s.height {
        for col in 0..s.width {
            let p = s.get_pixel(row, col);
            // WARNING rust-bmp params are (x, y)
            img.put_pixel(col as u32, row as u32, Rgb([p.r, p.g, p.b]));
        }
    }

    // 10 - 15 ms
    println!(
        "Transformed image to other type: {}",
        instant.elapsed().as_millis()
    );

    img.save_with_format("test.bmp", image::ImageFormat::Bmp)
        .unwrap();
    println!(
        "Saved second image after: {}",
        instant.elapsed().as_millis()
    ); // 25 ms

    img.save_with_format("test2.png", image::ImageFormat::Png)
        .unwrap();
    println!("Saved first image after: {}", instant.elapsed().as_millis()); // 45 - 55 ms

    image::save_buffer(
        "test.png",
        s.as_ref(),
        s.width as u32,
        s.height as u32,
        image::ColorType::Rgba8, // RGBA(8),
    )
    .unwrap();
    println!("Saved third image after: {}", instant.elapsed().as_millis()); // 45 - 55 ms
}
