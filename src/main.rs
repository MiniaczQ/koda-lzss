mod core;
mod encoder;
mod rand_source;
mod encoder_reader;
mod utility;
mod index_offset;
mod bit_writer;

use std::fs::File;

use encoder::LzssOptions;
use rand_source::RandomSource;

// add clap
// -i input file
// -o output file

fn main() {
    //let vector_input = vec![0, 1, 2, 3, 4, 5, 6, 7];
    //let random_input = RandomSource::chacha20_from_seed(123);
    let mut file_input = File::open("input_file").unwrap();
    let mut file_output = File::create("output_file").unwrap();

    let lzss = LzssOptions::new(8, 8);
    let _ = lzss.encode(&mut file_input, &mut file_output);
}
