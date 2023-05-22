mod bit_writer;
mod encoder;
mod encoder_reader;
mod index_offset;
mod rand_source;
mod utility;

use clap::Parser;
use encoder::LzssOptions;
use std::{fs::File, path::PathBuf};
//use rand_source::RandomSource;

// add clap
// -i input file
// -o output file

/// LZSS encoder
#[derive(Parser, Debug)]
#[command()]
struct Args {
    /// Input file
    #[arg(short, long)]
    input: PathBuf,
    // Output fiile
    #[arg(short, long)]
    output: PathBuf,
}

fn main() {
    let args = Args::parse();
    let mut file_input = File::open(args.input).expect("Problem with input file");
    let mut file_output = File::create(args.output).expect("Problem with output file");
    let lzss = LzssOptions::new(256, 16);
    lzss.encode(&mut file_input, &mut file_output).unwrap();
}
