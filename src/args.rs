use clap::Parser;

#[derive(Parser)]
pub struct Args {
    pub input: String,

    pub output: String,

    #[arg(short, long)]
    pub sort_name: bool,
}

