use std::{
    io::{self, Read, Write},
    mem::size_of,
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

    pub fn encode(
        &self,
        source: &mut impl Read,
        destination: &mut impl Write,
    ) -> io::Result<usize> {
        let mut destination = BitWriter::new(Box::new(destination));
        let mut buffer = EncoderReader::new(source, self.dict_size, self.buff_size)?;
        LzssSymbol::S(buffer[0]).write(&mut destination)?;

        let mut written = 0;
        while let Some(newly_written) = self.encode_one(&mut buffer, &mut destination)? {
            written += newly_written;
        }
        written += destination.flush()?;

        Ok(written)
    }

    fn encode_one(
        &self,
        buffer: &mut EncoderReader,
        destination: &mut impl BitWrite,
    ) -> io::Result<Option<usize>> {
        let remaining_buffer = self.buff_size - buffer.missing();
        if remaining_buffer == 0 {
            return Ok(None);
        }

        let (start, size) = find_largest_subset(
            buffer,
            self.dict_size,
            &IndexMapper::new(buffer, self.dict_size),
            remaining_buffer,
        );

        Ok(if LzssSymbol::PC_BIT_SIZE < size * 8 {
            let written = LzssSymbol::PC(start as u8, size as u8).write(destination)?;
            buffer.load(size)?;
            Some(written)
        } else {
            let written = LzssSymbol::S(buffer[self.dict_size]).write(destination)?;
            buffer.load(1)?;
            Some(written)
        })
    }
}

enum LzssSymbol {
    S(u8),
    PC(u8, u8),
}

impl LzssSymbol {
    const S_BIT_SIZE: usize = 9;
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
