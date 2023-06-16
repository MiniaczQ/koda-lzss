use std::{
    fmt::Debug,
    io::{self, Read},
    ops::Index,
};

pub struct EncoderReader<'a, Reader> {
    source: &'a mut Reader,
    size: usize,
    buffer: Vec<u8>,
    offset: usize,
    missing: usize,
}

impl<'a, Reader> EncoderReader<'a, Reader>
where
    Reader: Read + Debug,
{
    pub fn new(source: &'a mut Reader, dict_size: usize, buff_size: usize) -> io::Result<Self> {
        let size = dict_size + buff_size;
        // memory layout
        // [dictionary][input]
        let mut buff = vec![0u8; size];
        // first character to dictionary
        // also fill input buffer
        // [dictionary][input]
        // ___________12345678
        source.read_exact(&mut buff[dict_size - 1..dict_size])?;
        let write_buffer = &mut buff[dict_size..];
        let actual = Self::load_n_or_eof(source, write_buffer)?;
        let missing = write_buffer.len() - actual;
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

    fn load_n_or_eof(source: &mut impl Read, dest: &mut [u8]) -> io::Result<usize> {
        let mut read_total = 0;
        loop {
            let read = source.read(&mut dest[read_total..])?;
            if read == 0 {
                break;
            }
            read_total += read;
        }
        Ok(read_total)
    }

    fn load_to_end(&mut self, target: usize) -> io::Result<usize> {
        assert_ne!(0, target, "a");
        let write_buffer = &mut self.buffer[self.offset..];
        assert_eq!(write_buffer.len(), target);
        let actual = Self::load_n_or_eof(self.source, write_buffer)?;
        self.missing += target - actual;
        self.offset += target;
        self.offset %= self.size;
        Ok(actual)
    }

    fn load_from_start(&mut self, target: usize) -> io::Result<usize> {
        let write_buffer = &mut self.buffer[self.offset..self.offset + target];
        assert_eq!(write_buffer.len(), target);
        let actual = Self::load_n_or_eof(self.source, write_buffer)?;
        self.missing += target - actual;
        self.offset += target;
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

impl<'a, Reader> Index<usize> for EncoderReader<'a, Reader> {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        if self.size < index {
            panic!("Index out of bounds")
        } else {
            &self.buffer[(self.offset + index) % self.size]
        }
    }
}

impl<'a, T> Debug for EncoderReader<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncoderReader")
            .field("size", &self.size)
            .field("buffer", &self.buffer)
            .field("offset", &self.offset)
            .field("missing", &self.missing)
            .finish()
    }
}
