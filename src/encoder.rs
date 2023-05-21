use std::{
    collections::VecDeque,
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
        while let Ok(newly_written) = self.encode_one(&mut buffer, &mut destination) {
            written += newly_written
        }

        Ok(written)
    }

    fn encode_one(
        &self,
        buffer: &mut EncoderReader,
        destination: &mut impl BitWrite,
    ) -> io::Result<usize> {
        let (start, size) = find_largest_subset(
            buffer,
            self.dict_size,
            &IndexMapper::new(buffer, self.dict_size),
            self.buff_size,
        );

        Ok(if LzssSymbol::PC_SIZE < size {
            LzssSymbol::PC(start as u8, size as u8).write(destination)?;
            buffer.load(size)?;
            LzssSymbol::PC_SIZE
        } else {
            LzssSymbol::S(buffer[0]).write(destination)?;
            buffer.load(1)?;
            LzssSymbol::S_SIZE
        })
    }
}

enum LzssSymbol {
    S(u8),
    PC(u8, u8),
}

impl LzssSymbol {
    const S_SIZE: usize = 1 + size_of::<u8>();
    const PC_SIZE: usize = 1 + 2 * size_of::<u8>();

    fn write(&self, destination: &mut impl BitWrite) -> io::Result<()> {
        match *self {
            LzssSymbol::S(s) => {
                destination.write_bit(false)?;
                destination.write_byte(s);
                Ok(())
            }
            LzssSymbol::PC(p, c) => {
                destination.write_bit(true)?;
                destination.write_byte(p)?;
                destination.write_byte(c)?;
                Ok(())
            }
        }
    }

    fn read(source: &mut impl Read) -> io::Result<Self> {
        let mut variant = [0u8; size_of::<u8>()];
        source.read_exact(&mut variant)?;
        Ok(match variant[0] {
            0 => {
                let mut s = [0u8; size_of::<u8>()];
                source.read_exact(&mut s)?;
                Self::S(s[0])
            }
            _ => {
                let mut p = [0u8; size_of::<u8>()];
                source.read_exact(&mut p)?;
                let mut c = [0u8; size_of::<u8>()];
                source.read_exact(&mut c)?;
                Self::PC(u8::from_be_bytes(p), u8::from_be_bytes(c))
            }
        })
    }
}
