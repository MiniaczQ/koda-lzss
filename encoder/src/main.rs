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
struct RawArgs {
    /// Input file
    #[arg()]
    input: PathBuf,
    // Output fiile
    #[arg()]
    output: PathBuf,
    /// Dictionary indexing bits
    #[arg(short, long, default_value_t = 8)]
    dictionary_bits: u8,
    /// Dictionary size bits
    #[arg(short, long, default_value_t = 8)]
    max_match_bits: u8,
    /// Dictionary buffer size
    #[arg(long)]
    dictionary_size: Option<u32>,
    /// Max match size
    #[arg(long)]
    max_match_size: Option<u32>,
}

struct Args {
    input: PathBuf,
    output: PathBuf,
    dictionary_bits: u8,
    max_match_bits: u8,
    dictionary_size: usize,
    max_match_size: usize,
}

fn args() -> Result<Args, String> {
    let args: RawArgs = RawArgs::parse();
    if !args.input.exists() {
        return Err("Input file does not exist.".to_owned());
    }
    if args.output.exists() {
        println!("Output file already exists and will be overwritten.");
    }
    if args.dictionary_bits < 1 || args.dictionary_bits > 30 {
        return Err("Dictionary bits have to be in range [1..30]".to_owned());
    }
    if args.max_match_bits < 1 || args.max_match_bits > 30 {
        return Err("Max match bits have to be in range [1..30]".to_owned());
    }
    let max_dictionary_size = 2 ^ args.dictionary_bits as u32;
    let dictionary_size = args.dictionary_size.unwrap_or(max_dictionary_size);
    if dictionary_size < 1 || dictionary_size > max_dictionary_size {
        return Err(format!(
            "Dictionary size has to be in range [1..{}]",
            max_dictionary_size
        ));
    }
    let max_max_match_size = 2 ^ args.max_match_bits as u32;
    let max_match_size = args.max_match_size.unwrap_or(max_max_match_size);
    if max_match_size < 1 || max_match_size > max_max_match_size {
        return Err(format!(
            "Max match size has to be in range [1..{}]",
            max_max_match_size
        ));
    }
    Ok(Args {
        input: args.input,
        output: args.output,
        dictionary_bits: args.dictionary_bits,
        max_match_bits: args.max_match_bits,
        dictionary_size: dictionary_size as usize,
        max_match_size: max_match_size as usize,
    })
}

fn main() {
    let args = args().unwrap();
    let mut file_input = File::open(args.input).unwrap();
    let mut file_output = File::create(args.output).unwrap();
    let lzss = LzssOptions::new(args.dictionary_size, args.max_match_size);
    let (read, written) = lzss.encode(&mut file_input, &mut file_output).unwrap();
    println!("Compressed {} bytes into {}.", read, written);
}
