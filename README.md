Simple LZSS implementation

# Encoder (in Rust)

```sh
cargo run --bin encoder -- -i <input file> -o <output file>
```

```
LZSS encoder

Usage: encoder.exe [OPTIONS] <INPUT> <OUTPUT>

Arguments:
  <INPUT>   Input file
  <OUTPUT>  Output file

Options:
  -d, --dictionary-bits <DICTIONARY_BITS>  Dictionary indexing bits [default: 8]
  -m, --max-match-bits <MAX_MATCH_BITS>    Dictionary size bits [default: 8]
      --dictionary-size <DICTIONARY_SIZE>  Dictionary buffer size
      --max-match-size <MAX_MATCH_SIZE>    Max match size
  -e                                       Can matches extend into the input buffer
  -h, --help                               Print help
```

# Decoder (in Python)

Basic usage:

```sh
python py/decoder.py <input file> <output file>
```

Run

```sh
python py/decoder.py --help
```

to see accepted command line arguments:

```
usage: decoder.py [-h] [--window-size WINDOW_SIZE] [--length-width LENGTH_WIDTH] [--length-bias LENGTH_BIAS]
                  [--distance-width DISTANCE_WIDTH] [--flag-width FLAG_WIDTH] [--invert-flag] [--back-distance]
                  [input_file] [output_file]

LZSS sliding window decoder

positional arguments:
  input_file            Input file (to be decoded)
  output_file           Output file (target)

options:
  -h, --help            show this help message and exit
  --window-size WINDOW_SIZE, -w WINDOW_SIZE
                        Sliding window size (in bytes)
  --length-width LENGTH_WIDTH, -l LENGTH_WIDTH
                        Reference length width (in bits)
  --length-bias LENGTH_BIAS, -b LENGTH_BIAS
                        Reference length bias
  --distance-width DISTANCE_WIDTH
                        Reference distance width (in bits); zero means auto
  --flag-width FLAG_WIDTH
                        Flag width (in bits)
  --invert-flag         Treat zero as literal flag and others as reference flag
  --back-distance       Count distance from the end of the window
```
