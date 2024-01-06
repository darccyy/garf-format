use clap::{builder::NonEmptyStringValueParser, Parser};

#[derive(Parser)]
pub struct Args {
    /// Input file or directory
    pub input: String,

    /// Output file or directory
    pub output: String,

    /// Watermark text to add to image
    #[arg(short, long, required = true, value_parser = NonEmptyStringValueParser::new())]
    pub watermark: String,

    /// Whether to sort names of files before converting
    #[arg(short, long)]
    pub sort_name: bool,

    /// Adjust x-position of 2/3rds cutoff, as percent of comic width
    #[arg(short, long)]
    pub twothirds_adjust: Option<f32>,
}
