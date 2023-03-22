use std::{
    collections::VecDeque,
    io::{self, Read, Write},
    mem::size_of,
};

use crate::{
    encoder_reader::{EncoderReader},
    index_offset::IndexOffset, utility::find_largest_subset,
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
        let mut buffer = EncoderReader::new(source, self.dict_size, self.buff_size)?;
        LzssSymbol::S(buffer[0]).write(destination)?;

        let mut written = 0;
        while let Ok(newly_writter) = self.encode(source, destination) {
            written += newly_writter
        }

        Ok(written)
    }

    fn encode_one(
        &self,
        buffer: &mut EncoderReader,
        destination: &mut impl Write,
    ) -> io::Result<usize> {
        let (start, size) = find_largest_subset(
            buffer,
            self.dict_size,
            &IndexOffset::new(buffer, self.dict_size),
            self.buff_size,
        );

        Ok(if LzssSymbol::PC_SIZE < size {
            LzssSymbol::PC(start as u64, size as u64).write(destination)?;
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
    PC(u64, u64),
}

impl LzssSymbol {
    const S_SIZE: usize = size_of::<u8>() * 2;
    const PC_SIZE: usize = size_of::<u8>() + size_of::<u64>() * 2;

    fn write(&self, destination: &mut impl Write) -> io::Result<()> {
        match *self {
            LzssSymbol::S(s) => destination.write_all(&[0u8, s]),
            LzssSymbol::PC(p, c) => {
                destination.write_all(&[1u8])?;
                destination.write_all(&p.to_be_bytes())?;
                destination.write_all(&c.to_be_bytes())
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
                let mut p = [0u8; size_of::<u64>()];
                source.read_exact(&mut p)?;
                let mut c = [0u8; size_of::<u64>()];
                source.read_exact(&mut c)?;
                Self::PC(u64::from_be_bytes(p), u64::from_be_bytes(c))
            }
        })
    }
}
