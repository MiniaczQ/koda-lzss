mod bit_writer;
mod encoder;
mod encoder_reader;
mod index_offset;
mod utility;

use clap::Parser;
use encoder::LzssOptions;
use std::{fs::File, path::PathBuf};

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
    /// Dict size (max 256)
    #[arg(short, long, default_value_t = 256)]
    dict_size: usize,
    /// Input buffer size (max 256)
    #[arg(short, long, default_value_t = 16)]
    buffer_size: usize,
}

fn main() {
    let args = Args::parse();
    let mut file_input = File::open(args.input).expect("Problem with input file");
    let mut file_output = File::create(args.output).expect("Problem with output file");
    assert!(
        args.dict_size <= 256,
        "This implementation only supports up to 255 dictionary addresses"
    );
    assert!(
        args.buffer_size <= 256,
        "This implementation only supports up to 255 input buffeer addresses"
    );
    let lzss = LzssOptions::new(args.dict_size, args.buffer_size);
    let (read, written) = lzss.encode(&mut file_input, &mut file_output).unwrap();
    println!("Compressed {} bytes into {}.", read, written);
}
