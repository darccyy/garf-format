use std::fs;

use image::DynamicImage;
use image::GenericImageView;
use image::ImageBuffer;
use image::Rgba;

fn main() {
    let icon = image::open("watermark.png").expect("open icon image");

    let files = fs::read_dir("in").expect("read dir").flatten();

    for file in files {
        let filename = file.file_name();
        let filename = filename.to_string_lossy().to_string();

        println!("{filename}");

        let original = image::open(format!("in/{filename}")).expect("open image");

        let Some(composed) = compose_image(&icon, &original) else { continue };

        composed
            .save(format!("out/{filename}"))
            .expect("save image");
    }
}

fn compose_image(
    icon: &DynamicImage,
    original: &DynamicImage,
) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let dimensions = original.dimensions();
    let original_width = dimensions.0;
    let original_height = dimensions.1;

    println!("Original dimensions: {original_width} x {original_height}");

    let ratio = original_width as f32 / original_height as f32;
    println!("Aspect ratio: {ratio}");

    if ratio < 1.5 {
        println!("SUNDAY");
        // return None;
    }

    let new_width = original_width * 2 / 3 - 12;
    let new_height = original_height * 2;

    println!("New dimensions: {new_width} x {new_height}");

    icon.resize(new_width, new_height, image::imageops::FilterType::Nearest);

    let mut composed = ImageBuffer::new(new_width, new_height);

    for y in 0..new_height {
        for x in 0..new_width {
            let original_x = x % original_width;
            let original_y = y % original_height;

            let original_x = if y < original_height {
                original_x
            } else {
                original_x + new_width + 4
            };

            let pixel = if original_x < original_width {
                get_pixel_checked(original, original_x, original_y)
            } else {
                get_pixel_checked(
                    icon,
                    (x + 10 - new_width / 2) * 2,
                    (y + 30 - original_height) * 2,
                )
            };

            let pixel = pixel.unwrap_or(Rgba([0, 0, 0, 0]));

            composed.put_pixel(x, y, Rgba(pixel.0));
        }
    }

    Some(composed)
}

fn get_pixel_checked(image: &DynamicImage, x: u32, y: u32) -> Option<Rgba<u8>> {
    let (width, height) = image.dimensions();

    if x < width && y < height {
        Some(image.get_pixel(x, y))
    } else {
        None
    }
}
