mod args;

use clap::Parser;
use std::{fs, path::Path};

use args::Args;
use comic_format::convert_image;

fn main() {
    // Default stuff
    let icon = image::load_from_memory(include_bytes!("../icon.png")).expect("open icon image");

    // Get input and output files/directories from arguments
    let args = Args::parse();
    let input = &args.input;
    let output = &args.output;

    // Check input file/directory exists
    if !Path::new(input).exists() {
        panic!("in file/directory not exist");
    }

    // Convert single file, if input is file
    if Path::new(input).is_file() {
        convert_and_save(input, output, &icon, &args.watermark);
        return;
    }

    // Convert all files in directory, if input is directory
    let mut files: Vec<_> = fs::read_dir(input).expect("read dir").flatten().collect();
    // Sort files by name
    if args.sort_name {
        files.sort_by_key(|file| {
            let filename = file.file_name();
            let filename = filename.to_string_lossy().to_string();
            filename
        });
    }

    // Create output directory if not exist
    if !Path::new(output).exists() {
        fs::create_dir_all(output).expect("create out dir");
    }

    // Convert each file in directory
    for file in files {
        let filename = file.file_name();
        let filename = filename.to_string_lossy().to_string();

        convert_and_save(
            &file.path().to_string_lossy(),
            &format!("{output}/{filename}"),
            &icon,
            &args.watermark,
        );
    }
}

/// Read an image, convert to output, and write
fn convert_and_save(input: &str, output: &str, icon: &image::DynamicImage, watermark: &str) {
    println!("{input} -> {output}");
    let image_in = image::open(input).expect("open image");
    let image_out = convert_image(image_in, icon, watermark);
    image_out.save(output).expect("save image");
}
