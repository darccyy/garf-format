use std::fs;

use comic::{add_padding, add_watermark, make_square, remove_padding};

fn main() {
    let dir_in = "/home/darcy/pics/garfield";
    let dir_out = "out";
    let sort_files = false;
    let watermark = "@garfieldeo@mastodon.world";

    let icon = image::open("icon.png").expect("open icon image");
    let mut files: Vec<_> = fs::read_dir(dir_in).expect("read dir").flatten().collect();

    if sort_files {
        files.sort_by_key(|file| {
            let filename = file.file_name();
            let filename = filename.to_string_lossy().to_string();
            filename
        });
    }

    for file in files {
        let filename = file.file_name();
        let filename = filename.to_string_lossy().to_string();

        println!("{filename}");

        let original = image::open(file.path()).expect("open image");

        let cropped = remove_padding(original);

        let squared = make_square(cropped, &icon);

        let padded = add_padding(squared);

        let watermarked = add_watermark(padded, watermark);

        watermarked
            .save(format!("{dir_out}/{filename}"))
            .expect("save image");
    }
}
