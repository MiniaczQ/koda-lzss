mod core;
mod encoder;
mod rand_source;
mod encoder_reader;
mod utility;
mod index_offset;

use std::fs::File;

use rand_source::RandomSource;

fn main() {
    let vector_input = vec![0, 1, 2, 3, 4, 5, 6, 7];
    let random_input = RandomSource::chacha20_from_seed(123);
    let file_input = File::open("input_file").unwrap();
}
