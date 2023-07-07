use clap::Parser;
use std::error::Error;

/// A Rust implementation of GZIP compression
#[derive(Debug, Parser)]
struct Cli {
    /// The input file to be compressed
    input: String,
    /// The output compressed file
    #[arg(short, long)]
    output: Option<String>
}
fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    rustgzip::compress_to_gzip(&args.input, &args.output.unwrap_or("a.out".to_string()))
}
