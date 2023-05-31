use std::io::Write;

use bitvec::{
    domain::Domain,
    prelude::{BitOrder, Msb0},
    slice::BitSlice,
    vec::BitVec,
};

pub trait BitWrite {
    fn write_bit(&mut self, bit: bool) -> std::io::Result<usize>;
    fn write_bits(&mut self, bits: &BitSlice) -> std::io::Result<usize>;
    fn end_flush(&mut self) -> std::io::Result<usize>;
}

pub struct BitWriter<Writer: Write> {
    inner: Writer,
    buffer: BitVec<usize, Msb0>,
}

impl<Writer: Write> BitWriter<Writer> {
    pub fn new(writer: Writer) -> Self {
        Self {
            inner: writer,
            buffer: BitVec::new(),
        }
    }
}

const AUTOFLUSH: usize = 1024;

impl<Writer: Write> BitWriter<Writer> {
    fn full_bytes(&self) -> usize {
        self.buffer.len() / 8
    }

    fn flush(&mut self) -> std::io::Result<usize> {}

    fn autoflush(&mut self) -> std::io::Result<()> {
        let full_bytes = self.full_bytes();
        if full_bytes > AUTOFLUSH {
            if let Domain::Region {
                head: None,
                body,
                tail: _,
            } = self.buffer.domain()
            {
                self.inner.
                self.inner.write_all(body.iter().map(|v| v.to_be_bytes()).flatten())?;
            }
        }
        Ok(())
    }
}

impl<Writer: Write> BitWrite for BitWriter<Writer> {
    fn write_bit(&mut self, bit: bool) -> std::io::Result<usize> {
        self.buffer.push(bit);
    }

    fn write_bits(&mut self, bits: &BitSlice) -> std::io::Result<usize> {
        todo!()
    }

    fn end_flush(&mut self) -> std::io::Result<usize> {
        todo!()
    }
}
