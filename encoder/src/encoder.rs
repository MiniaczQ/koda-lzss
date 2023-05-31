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
    dict_size: usize,
    buff_size: usize,
}

impl LzssOptions {
    pub fn new(dict_size: usize, buff_size: usize) -> Self {
        Self {
            dict_size,
            buff_size,
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
        let mut buffer = EncoderReader::<Reader>::new(source, self.dict_size, self.buff_size)?;
        LzssSymbol::S(buffer[0]).write(&mut destination)?;

        let mut read_total = self.buff_size - buffer.missing();
        let mut written_total = 0;
        loop {
            let (read, written) = self.encode_one(&mut buffer, &mut destination)?;
            read_total += read;
            written_total += written;

            if self.buff_size == buffer.missing() && read == 0 {
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
            self.dict_size,
            &IndexMapper::new(buffer, self.dict_size),
            self.buff_size - buffer.missing(),
        );

        let mut read = 0;
        let mut written = 0;

        if LzssSymbol::PC_BIT_SIZE < size * 8 {
            read += buffer.load(size)?;
            written += LzssSymbol::PC(start as u8, size as u8).write(destination)?;
        } else {
            let symbol = buffer[self.dict_size];
            read += buffer.load(1)?;
            written += LzssSymbol::S(symbol).write(destination)?;
        };

        Ok((read, written))
    }
}

enum LzssSymbol {
    S(u8),
    PC(u8, u8),
}

impl LzssSymbol {
    const PC_BIT_SIZE: usize = 17;

    fn write(&self, destination: &mut impl BitWrite) -> io::Result<usize> {
        match *self {
            LzssSymbol::S(s) => Ok(destination.write_bit(false)? + destination.write_byte(s)?),
            LzssSymbol::PC(p, c) => Ok(destination.write_bit(true)?
                + destination.write_byte(p)?
                + destination.write_byte(c)?),
        }
    }
}
