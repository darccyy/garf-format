use std::ops::RangeInclusive;

use image::{imageops, DynamicImage, GenericImageView, ImageBuffer, Rgba};
use imageproc::drawing::{draw_text_mut, text_size};
use rand::Rng;
use rusttype::{Font, Scale};

/// Maximum aspect ration (w/h) an image can be to be treated as a sunday comic
const MAX_SUNDAY_ASPECT_RATIO: f32 = 2.0;
/// Minimum value of a color to be considered white.
/// Used to crop initial padding, which can be any amount
const MIN_WHITE_THRESHOLD: u8 = 100;
/// Relative x-position for end of second panel (left of 2/3 divider) in non-Sunday comic.
const POS_TWOTHIRD_LEFT_AMOUNT: f32 = 0.655;
/// Relative x-position for start of third panel (right of 2/3 divider) in non-Sunday comic.
const POS_TWOTHIRD_RIGHT_AMOUNT: f32 = 0.666;
/// How much to increase the height of a non-Sunday comic, to make square
const POS_HEIGHT_AMOUNT: f32 = 2.03;
/// Relative x-position of watermark icon
const POS_ICON_X: f32 = 0.515;
/// Relative y-position of watermark icon
const POS_ICON_Y: f32 = 0.505;
/// Which resizing filter to use when scaling image.
/// Very slow! But who cares!
/// This filter should make the image look best
const RESIZE_FILTER: imageops::FilterType = imageops::FilterType::Lanczos3;
/// Relative amount of extra white to add
const PADDING_AMOUNT: f32 = 0.009;
/// White color for text and padding
const WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);
/// Black color for text
const BLACK: Rgba<u8> = Rgba([0, 0, 0, 255]);
/// Relative size of text, randomly-selected from range
const TEXT_SIZE: RangeInclusive<f32> = 0.03..=0.04;
/// Relative horizontal scale of text, randomly-selected from range
const TEXT_WIDTH_SCALE: RangeInclusive<f32> = 0.6..=1.1;
/// Relative width for text stroke
const TEXT_STROKE_WEIGHT: f32 = 0.09;
/// Relative positions for watermark icon bounds, for non-Sunday comic.
/// [LEFT, RIGHT, TOP, BOTTOM]
const EDGES_NORMAL: [f32; 4] = [0.52, 0.99, 0.51, 0.99];
/// Relative positions for watermark icon bounds, for Sunday comic.
/// [LEFT, RIGHT, TOP, BOTTOM]
const EDGES_SUNDAY: [f32; 4] = [0.01, 0.99, 0.71, 0.99];
/// Final width of image.
/// Height based on this too
const FINAL_WIDTH: u32 = 1200;

/// Conver image, including all operations
pub fn convert_image(image: DynamicImage, icon: &DynamicImage, watermark: &str) -> DynamicImage {
    let image = remove_padding(image);
    let image = make_square(image, &icon);
    let image = add_padding(image);
    let image = resize_image(image);
    let image = add_watermark(image, watermark);
    image
}

/// Remove initial white padding from image
fn remove_padding(mut image: DynamicImage) -> DynamicImage {
    let (width, height) = image.dimensions();

    let (mut min_x, mut min_y, mut max_x, mut max_y) = (width, height, 0, 0);

    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y);

            if !is_white_enough(pixel) {
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

/// Make comic fit in square image.
///
///  - Sunday does not change, just watermark added at bottom to make square.
///  - Non-Sunday wraps 3rd panel below, to make 2x2 grid, watermark is added in last square.
fn make_square(image: DynamicImage, icon: &DynamicImage) -> DynamicImage {
    let (width, height) = image.dimensions();

    let ratio = width as f32 / height as f32;

    let square = if ratio < MAX_SUNDAY_ASPECT_RATIO {
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
        let twothirds_left = (width as f32 * POS_TWOTHIRD_LEFT_AMOUNT) as u32;
        let twothirds_right = (width as f32 * POS_TWOTHIRD_RIGHT_AMOUNT) as u32;

        let square_width = twothirds_left;
        let square_height = (height as f32 * POS_HEIGHT_AMOUNT) as u32;

        let mut square = ImageBuffer::from_pixel(square_width, square_height, WHITE);

        imageops::overlay(&mut square, &image.to_rgba8(), 0, 0);

        imageops::overlay(
            &mut square,
            &image.to_rgba8(),
            -(twothirds_right as i64),
            (square_height - height) as i64,
        );

        let size = square_width.max(square_height) / 2;
        let icon = icon.resize(size, size, RESIZE_FILTER);

        imageops::overlay(
            &mut square,
            &icon.to_rgba8(),
            (twothirds_right as f32 * POS_ICON_X) as i64,
            (square_height as f32 * POS_ICON_Y) as i64,
        );

        square
    };

    DynamicImage::ImageRgba8(square)
}

/// Add extra white padding to image
fn add_padding(image: DynamicImage) -> DynamicImage {
    let (width, height) = image.dimensions();

    let padding = (width.min(height) as f32 * PADDING_AMOUNT) as u32;

    let mut padded = ImageBuffer::from_pixel(width + padding * 2, height + padding * 2, WHITE);

    imageops::overlay(&mut padded, &image, padding as i64, padding as i64);

    DynamicImage::ImageRgba8(padded)
}

/// Add watermark text to image
fn add_watermark(image: DynamicImage, text: &str) -> DynamicImage {
    let mut image = image.to_rgba8();

    let (width, height) = image.dimensions();

    let font = Vec::from(include_bytes!("../font.ttf") as &[u8]);
    let font = Font::try_from_vec(font).unwrap();

    let mut rng = rand::thread_rng();

    // Get random text size
    let text_height = width.max(height) as f32 * rng.gen_range(TEXT_SIZE);
    // Offset for stroke drawing
    let offset = (text_height * TEXT_STROKE_WEIGHT) as i32;

    let scale = Scale {
        // Distort (squish) to make shorter/longer
        x: text_height * rng.gen_range(TEXT_WIDTH_SCALE),
        y: text_height,
    };

    // Edges of icon
    let edges = if width != height {
        EDGES_NORMAL
    } else {
        EDGES_SUNDAY
    };

    // Relative to image size
    let (w, h) = text_size(scale, &font, text);
    let min_x = (width as f32 * edges[0]) as i32;
    let max_x = (width as f32 * edges[1]) as i32 - w;
    let min_y = (height as f32 * edges[2]) as i32;
    let max_y = (height as f32 * edges[3]) as i32 - h;
    // Prevent inverted ranges
    let max_x = min_x.max(max_x);
    let max_y = min_y.max(max_y);

    // Get random position
    let x = rng.gen_range(min_x..=max_x);
    let y = rng.gen_range(min_y..=max_y);

    const DIRECTIONS: [(i32, i32); 8] = [
        // Diagonals
        (-1, -1),
        (1, -1),
        (-1, 1),
        (1, 1),
        // Cardinals
        (0, -1),
        (-1, 0),
        (1, 0),
        (0, 1),
    ];
    for (dir_x, dir_y) in DIRECTIONS {
        draw_text_mut(
            &mut image,
            BLACK,
            x + offset * dir_x,
            y + offset * dir_y,
            scale,
            &font,
            text,
        );
    }

    draw_text_mut(&mut image, WHITE, x, y, scale, &font, text);

    DynamicImage::ImageRgba8(image)
}

/// Returns if pixel value is considered white enough (MIN_WHITE_THRESHOLD)
fn is_white_enough(pixel: Rgba<u8>) -> bool {
    let Rgba([r, g, b, a]) = pixel;
    if a < 255 {
        return true;
    }
    r >= MIN_WHITE_THRESHOLD && g >= MIN_WHITE_THRESHOLD && b >= MIN_WHITE_THRESHOLD
}

/// Resize image to make width equal final width, without changing aspect ratio
fn resize_image(image: DynamicImage) -> DynamicImage {
    let (width, height) = image.dimensions();
    let ratio = width as f32 / height as f32;
    let final_height = (ratio * FINAL_WIDTH as f32) as u32;

    image.resize(FINAL_WIDTH, final_height, RESIZE_FILTER)
}
