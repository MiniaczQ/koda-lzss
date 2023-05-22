use std::{
    fmt::Display,
    io::{self, Read},
    ops::Index,
};

pub struct EncoderReader<'a> {
    source: &'a mut dyn Read,
    size: usize,
    buffer: Vec<u8>,
    offset: usize,
    missing: usize,
}

impl<'a> EncoderReader<'a> {
    pub fn new(source: &'a mut impl Read, dict_size: usize, buff_size: usize) -> io::Result<Self> {
        let size = dict_size + buff_size;
        // memory layout
        // [dictionary][input]
        let mut buff = vec![0u8; size];
        // first character to dictionary
        // also fill input buffer
        // [dictionary][input]
        // ___________12345678
        let write_buffer = &mut buff[dict_size - 1..];
        let n = source.read(write_buffer)?;
        let missing = write_buffer.len() - n;
        // fill dictionary
        // [dictionary][input]
        // 1111111111112345678
        let char = buff[dict_size - 1];
        buff[0..dict_size - 1].fill(char);
        Ok(Self {
            source,
            size,
            buffer: buff,
            offset: 0,
            missing,
        })
    }

    #[must_use]
    fn load_to_end(&mut self) -> io::Result<bool> {
        let write_buffer = &mut self.buffer[self.offset..];
        let n = self.source.read(write_buffer)?;
        self.missing += write_buffer.len() - n;
        if self.missing > 0 {
            self.offset += n;
            Ok(false)
        } else {
            self.offset = 0;
            Ok(true)
        }
    }

    #[must_use]
    fn load_from_start(&mut self, n: usize) -> io::Result<()> {
        let write_buffer = &mut self.buffer[self.offset..self.offset + n];
        let m = self.source.read(write_buffer)?;
        self.missing += n - m;
        self.offset += n;
        Ok(())
    }

    pub fn load(&mut self, n: usize) -> io::Result<()> {
        if self.size < n {
            panic!("Cannot read more bytes than buffer can contain");
        }
        let new_offs = self.offset + n;
        if new_offs > self.size {
            if !(self.load_to_end()?) {
                return Ok(());
            }
            self.load_from_start(new_offs % self.size)?;
        } else {
            self.load_from_start(n)?;
        }
        Ok(())
    }

    pub fn missing(&self) -> usize {
        self.missing
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
