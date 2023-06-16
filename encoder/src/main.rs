mod bit_writer;
mod encoder;
mod encoder_reader;
mod index_offset;
mod utility;

use clap::Parser;
use encoder::LzssOptions;
use std::{fs::File, io::BufReader, path::PathBuf};

/// LZSS encoder
#[derive(Parser, Debug)]
#[command()]
struct RawArgs {
    /// Input file
    #[arg()]
    input: PathBuf,
    /// Output file
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
    /// Can matches extend into the input buffer
    #[arg(short)]
    extend_into_input: bool,
}

struct Args {
    input: PathBuf,
    output: PathBuf,
    dictionary_bits: usize,
    max_match_bits: usize,
    dictionary_size: usize,
    max_match_size: usize,
    extend_into_input: bool,
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
    let max_dictionary_size = 2u32.pow(args.dictionary_bits as u32);
    let dictionary_size = args.dictionary_size.unwrap_or(max_dictionary_size);
    if dictionary_size < 1 || dictionary_size > max_dictionary_size {
        return Err(format!(
            "Dictionary size has to be in range [1..{}]",
            max_dictionary_size
        ));
    }
    let max_max_match_size = 2u32.pow(args.max_match_bits as u32);
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
        dictionary_bits: args.dictionary_bits as usize,
        max_match_bits: args.max_match_bits as usize,
        dictionary_size: dictionary_size as usize,
        max_match_size: max_match_size as usize,
        extend_into_input: args.extend_into_input,
    })
}

fn main() {
    let args = args().unwrap();
    let mut file_input = BufReader::new(File::open(args.input).unwrap());
    let mut file_output = File::create(args.output.clone()).unwrap();
    //let mut debug = BufWriter::new(File::create(args.output.with_extension("debug")).unwrap());
    let lzss = LzssOptions::new(
        args.dictionary_bits,
        args.max_match_bits,
        args.dictionary_size,
        args.max_match_size,
        args.extend_into_input,
    );
    let (read, written) = lzss
        .encode(&mut file_input, &mut file_output, Option::<&mut File>::None)
        .unwrap();
    println!("Compressed {} bytes into {} bytes.", read, written);
}
