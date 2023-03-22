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
        let mut buff = Vec::with_capacity(size);
        read.read_exact(&mut buff[0..1])?;
        let char = buff[0];
        buff[1..dict_size].fill(char);
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
