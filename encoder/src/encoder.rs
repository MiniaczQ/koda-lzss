use std::{
    fmt::Debug,
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
    ) -> io::Result<(usize, usize)>
    where
        Reader: Read + Debug,
    {
        let mut destination = BitWriter::new(Box::new(destination));
        let mut buffer =
            EncoderReader::<Reader>::new(source, self.dictionary_size, self.max_match_size)?;
        let mut written_total = self.write_symbol(LzssSymbol::S(buffer[0]), &mut destination)?;

        let mut read_total = self.max_match_size - buffer.missing() + 1;
        loop {
            let (read, written) = self.encode_one(&mut buffer, &mut destination)?;
            read_total += read;
            written_total += written;

            if self.max_match_size == buffer.missing() && read == 0 {
                break;
            }
        }
        written_total += destination.end_flush()?;

        Ok((read_total, written_total))
    }

    fn encode_one<Reader>(
        &self,
        buffer: &mut EncoderReader<Reader>,
        destination: &mut impl BitWrite,
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

        if self.dictionary_bits + self.max_match_bits + 1 < size * 8 {
            read += buffer.load(size)?;
            written += self.write_symbol(LzssSymbol::PC(start as u32, size as u32), destination)?;
        } else {
            let symbol = buffer[self.dictionary_size];
            read += buffer.load(1)?;
            written += self.write_symbol(LzssSymbol::S(symbol), destination)?;
        };

        Ok((read, written))
    }

    fn write_symbol(
        &self,
        symbol: LzssSymbol,
        destination: &mut impl BitWrite,
    ) -> io::Result<usize> {
        match symbol {
            LzssSymbol::S(s) => {
                //println!("char {:?}", s as char);
                Ok(destination.write_bit(false)? + destination.write_few(s as usize, 8)?)
            }
            LzssSymbol::PC(p, c) => {
                //println!("pair {:?} {:?}", p, c);
                Ok(destination.write_bit(true)?
                    + destination.write_few(p as usize, self.dictionary_bits)?
                    + destination.write_few(c as usize, self.max_match_bits)?)
            }
        }
    }
}

#[derive(Debug)]
enum LzssSymbol {
    S(u8),
    PC(u32, u32),
}
