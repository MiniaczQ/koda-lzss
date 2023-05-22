use std::{
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
    fn load_to_end(&mut self, target: usize) -> io::Result<usize> {
        assert_ne!(0, target, "a");
        let write_buffer = &mut self.buffer[self.offset..];
        assert_eq!(write_buffer.len(), target);
        let actual = self.source.read(write_buffer)?;
        self.missing += target - actual;
        self.offset += actual;
        self.offset %= self.size;
        Ok(actual)
    }

    #[must_use]
    fn load_from_start(&mut self, target: usize) -> io::Result<usize> {
        let write_buffer = &mut self.buffer[self.offset..self.offset + target];
        assert_eq!(write_buffer.len(), target);
        let actual = self.source.read(write_buffer)?;
        self.missing += target - actual;
        self.offset += actual;
        Ok(actual)
    }

    pub fn load(&mut self, target: usize) -> io::Result<usize> {
        assert!(target < self.size);
        let mut actual = 0;
        let next_offset = self.offset + target;
        let to_end = self.size - self.offset;
        if next_offset == self.size {
            actual += self.load_to_end(to_end)?;
        } else if next_offset >= self.size {
            actual += self.load_to_end(to_end)?;
            actual += self.load_from_start(target - to_end)?;
        } else {
            actual += self.load_from_start(target)?;
        }
        Ok(actual)
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
