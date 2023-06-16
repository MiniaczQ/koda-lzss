use std::{
    fmt::{Debug, Display},
    io::{self, Read, Write},
};

use crate::{
    bit_writer::{BitWrite, BitWriter},
    encoder_reader::EncoderReader,
    index_offset::IndexMapper,
    utility::find_largest_subset,
};

pub struct LzssOptions {
    dictionary_bits: usize,
    max_match_bits: usize,
    dictionary_size: usize,
    max_match_size: usize,
    extend_into_input: bool,
}

impl LzssOptions {
    pub fn new(
        dictionary_bits: usize,
        max_match_bits: usize,
        dictionary_size: usize,
        max_match_size: usize,
        extend_into_input: bool,
    ) -> Self {
        Self {
            dictionary_bits,
            max_match_bits,
            dictionary_size,
            max_match_size,
            extend_into_input,
        }
    }

    pub fn encode<Reader>(
        &self,
        source: &mut Reader,
        destination: &mut impl Write,
        mut debug: Option<&mut impl Write>,
    ) -> io::Result<(usize, usize)>
    where
        Reader: Read + Debug,
    {
        let mut destination = BitWriter::new(Box::new(destination));
        let mut buffer =
            EncoderReader::<Reader>::new(source, self.dictionary_size, self.max_match_size)?;
        let mut written_total = self.write_symbol(&LzssSymbol::S(buffer[0]), &mut destination)?;

        let mut read_total = self.max_match_size - buffer.missing() + 1;
        let mut i: u64 = 0;
        loop {
            let (read, written) =
                self.encode_one(&mut buffer, &mut destination, debug.as_mut(), i, read_total)?;
            read_total += read;
            written_total += written;

            if self.max_match_size == buffer.missing() && read == 0 {
                break;
            }

            i += 1;
        }
        written_total += destination.end_flush()?;

        Ok((read_total, written_total))
    }

    fn encode_one<Reader>(
        &self,
        buffer: &mut EncoderReader<Reader>,
        destination: &mut impl BitWrite,
        debug: Option<&mut impl Write>,
        i: u64,
        total_read: usize,
    ) -> io::Result<(usize, usize)>
    where
        Reader: Read + Debug,
    {
        let (start, size) = find_largest_subset(
            buffer,
            self.dictionary_size,
            &IndexMapper::new(buffer, self.dictionary_size),
            self.max_match_size - buffer.missing(),
            self.extend_into_input,
        );

        let mut read = 0;
        let mut written = 0;

        let symbol = if self.dictionary_bits + self.max_match_bits + 1 < size * 8 {
            read += buffer.load(size)?;
            LzssSymbol::PC(start as u32, size as u32)
        } else {
            let symbol = buffer[self.dictionary_size];
            read += buffer.load(1)?;
            LzssSymbol::S(symbol)
        };

        written += self.write_symbol(&symbol, destination)?;

        if let Some(debug) = debug {
            writeln!(debug, "#{} #{}", i, total_read).unwrap();
            writeln!(debug, " symbol:       {}", symbol).unwrap();
            writeln!(debug).unwrap();
        }

        Ok((read, written))
    }

    fn write_symbol(
        &self,
        symbol: &LzssSymbol,
        destination: &mut impl BitWrite,
    ) -> io::Result<usize> {
        match symbol {
            LzssSymbol::S(s) => {
                Ok(destination.write_bit(false)? + destination.write_few(*s as u32, 8)?)
            }
            LzssSymbol::PC(p, c) => {
                Ok(destination.write_bit(true)?
                    + destination.write_few(*p, self.dictionary_bits)?
                    + destination.write_few(*c, self.max_match_bits)?)
            }
        }
    }
}

#[derive(Debug)]
enum LzssSymbol {
    S(u8),
    PC(u32, u32),
}

impl Display for LzssSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LzssSymbol::S(c) => f.write_fmt(format_args!("(0, {}) {}", c, *c as char)),
            LzssSymbol::PC(a, b) => f.write_fmt(format_args!("(1, {}, {})", a, b)),
        }
    }
}
