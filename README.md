Simple LZSS implementation

# Encoder (in Rust)

```sh
cargo run -- -i <input file> -o <output file>
```

```
LZSS encoder

Usage: koda-lzss.exe [OPTIONS] --input <INPUT> --output <OUTPUT>

Options:
  -i, --input <INPUT>              Input file
  -o, --output <OUTPUT>
  -d, --dict-size <DICT_SIZE>      Dict size (max 256) [default: 256]
  -b, --buffer-size <BUFFER_SIZE>  Input buffer size (max 256) [default: 256]
  -h, --help                       Print help
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
