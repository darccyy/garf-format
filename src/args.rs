use clap::Parser;

#[derive(Parser)]
pub struct Args {
    /// Input file or directory
    pub input: String,

    /// Output file or directory
    pub output: String,

    /// Watermark text to add to image
    pub watermark: String,

    /// Whether to sort names of files before converting
    #[arg(short, long)]
    pub sort_name: bool,
}

