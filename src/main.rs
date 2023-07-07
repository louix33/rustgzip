use clap::Parser;

/// A Rust implementation of GZIP compression
#[derive(Debug, Parser)]
struct Cli {
    /// The input file to be compressed
    input: String,
    /// The output file
    #[arg(short, long)]
    output: Option<String>
}
fn main() {
    let args = Cli::parse();
    rustgzip::compress_to_gzip(&args.input, &args.output.unwrap_or("a.out".to_string())).unwrap();
}
