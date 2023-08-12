use std::fs;

use image::imageops;
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

        let cropped = remove_padding(original);

        let squared = make_square(cropped, &icon);

        let padded = add_padding(squared);

        padded.save(format!("out/{filename}")).expect("save image");
    }
}

fn add_padding(image: DynamicImage) -> DynamicImage {
    let (width, height) = image.dimensions();

    const PADDING_AMOUNT: f32 = 0.009;

    let padding = (width.min(height) as f32 * PADDING_AMOUNT) as u32;

    let mut padded = ImageBuffer::from_pixel(width + padding * 2, height + padding * 2, WHITE);

    imageops::overlay(&mut padded, &image, padding as i64, padding as i64);

    DynamicImage::ImageRgba8(padded)
}

const WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);

const RESIZE_FILTER: imageops::FilterType = imageops::FilterType::Gaussian;

fn make_square(image: DynamicImage, icon: &DynamicImage) -> DynamicImage {
    let (width, height) = image.dimensions();

    let ratio = width as f32 / height as f32;

    let square = if ratio < 2.0 {
        println!("  SUNDAY");

        let mut square = ImageBuffer::from_pixel(width, width, WHITE);

        image::imageops::overlay(&mut square, &image.to_rgba8(), 0, 0);

        let mut icon = icon.resize(width, width, RESIZE_FILTER);
        let icon = icon.crop(0, (width as f32 * 0.35) as u32, width, width);

        imageops::overlay(
            &mut square,
            &icon.to_rgba8(),
            0,
            (height as f32 * 1.01) as i64,
        );

        square
    } else {
        let twothird_left = (width as f32 * 0.655) as u32;
        let twothird_right = (width as f32 * 0.666) as u32;

        let square_width = twothird_left;
        let square_height = (height as f32 * 2.03) as u32;

        let mut square = ImageBuffer::from_pixel(square_width, square_height, WHITE);

        imageops::overlay(&mut square, &image.to_rgba8(), 0, 0);

        imageops::overlay(
            &mut square,
            &image.to_rgba8(),
            -(twothird_right as i64),
            (square_height - height) as i64,
        );

        let size = square_width.max(square_height) / 2;
        let icon = icon.resize(size, size, RESIZE_FILTER);

        imageops::overlay(
            &mut square,
            &icon.to_rgba8(),
            (twothird_right as f32 * 0.515) as i64,
            (square_height as f32 * 0.505) as i64,
        );

        square
    };

    DynamicImage::ImageRgba8(square)
}

fn remove_padding(mut image: DynamicImage) -> DynamicImage {
    let (width, height) = image.dimensions();

    let (mut min_x, mut min_y, mut max_x, mut max_y) = (width, height, 0, 0);

    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y);

            if !is_white(pixel) {
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
        }
    }

    if min_x <= max_x && min_y <= max_y {
        image.crop(min_x, min_y, max_x - min_x + 1, max_y - min_y + 1)
    } else {
        image
    }
}

fn is_white(pixel: Rgba<u8>) -> bool {
    let Rgba([r, g, b, a]) = pixel;

    if a < 255 {
        return true;
    }

    const MIN_VALUE: u8 = 100;

    r >= MIN_VALUE && g >= MIN_VALUE && b >= MIN_VALUE
}

#[allow(dead_code)]
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
