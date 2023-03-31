use std::{
    io::{self, Read},
    mem::swap,
    ops::Index,
};

pub struct EncoderReader<'a> {
    source: &'a mut dyn Read,
    size: usize,
    buffer: Vec<u8>,
    offset: usize,
}

impl<'a> EncoderReader<'a> {
    pub fn new(read: &'a mut impl Read, dict_size: usize, buff_size: usize) -> io::Result<Self> {
        let size = dict_size + buff_size;
        // memory layout
        // [dictionary][input]
        let mut buff = vec![0u8; size];
        // first character to dictionary
        // also fill input buffer
        // [dictionary][input]
        // ___________12345678
        read.read_exact(&mut buff[dict_size - 1..])?;
        // fill dictionary
        // [dictionary][input]
        // 1111111111112345678
        let char = buff[dict_size - 1];
        buff[0..dict_size - 1].fill(char);
        Ok(Self {
            source: read,
            size,
            buffer: buff,
            offset: 0,
        })
    }

    #[must_use]
    fn load_to_end(&mut self) -> io::Result<()> {
        self.source.read_exact(&mut self.buffer[self.offset..])?;
        self.offset = 0;
        Ok(())
    }

    #[must_use]
    fn load_from_start(&mut self, n: usize) -> io::Result<()> {
        self.source.read_exact(&mut self.buffer[..n])?;
        self.offset = n;
        Ok(())
    }

    pub fn load(&mut self, mut n: usize) -> io::Result<()> {
        if self.size < n {
            panic!("Cannot read more bytes than buffer can contain");
        }
        let new_offs = self.offset + n;
        if new_offs > self.size {
            self.load_to_end()?;
        }
        self.load_from_start(new_offs % self.size)?;
        Ok(())
    }
}

impl<'a> Index<usize> for EncoderReader<'a> {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        if self.size < index {
            panic!("Index out of bounds")
        } else {
            &self.buffer[(self.offset + index) % self.size]
        }
    }
}
