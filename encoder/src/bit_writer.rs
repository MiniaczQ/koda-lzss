use std::io::Write;

use bitvec::{domain::Domain, prelude::Msb0, slice::BitSlice, vec::BitVec};

pub trait BitWrite {
    fn write_bit(&mut self, bit: bool) -> std::io::Result<usize>;
    fn write_few(&mut self, bits: u32, n_bits: usize) -> std::io::Result<usize>;
    fn end_flush(&mut self) -> std::io::Result<usize>;
}

pub struct BitWriter<Writer: Write> {
    inner: Writer,
    buffer: BitVec<u8, Msb0>,
}

impl<Writer: Write> BitWriter<Writer> {
    pub fn new(writer: Writer) -> Self {
        Self {
            inner: writer,
            buffer: BitVec::new(),
        }
    }
}

const AUTOFLUSH: usize = 1024 * 1024;

impl<Writer: Write> BitWriter<Writer> {
    fn flush(&mut self) -> std::io::Result<usize> {
        let full_bytes = self.buffer.len() / 8;
        if let Domain::Region {
            head: None,
            body,
            tail: _,
        } = self.buffer.domain()
        {
            self.inner.write_all(
                &body
                    .iter()
                    .map(|v| v.to_be_bytes())
                    .flatten()
                    .collect::<Vec<_>>(),
            )?;
            self.buffer = self.buffer[full_bytes * 8..].to_bitvec();
            self.buffer.force_align();
            Ok(full_bytes)
        } else {
            unreachable!("BitVec in invalid state");
        }
    }

    fn autoflush(&mut self) -> std::io::Result<usize> {
        let full_bytes = self.buffer.len() / 8;
        if full_bytes > AUTOFLUSH {
            self.flush()
        } else {
            Ok(0)
        }
    }
}

impl<Writer: Write> BitWrite for BitWriter<Writer> {
    fn write_bit(&mut self, bit: bool) -> std::io::Result<usize> {
        self.buffer.push(bit);
        //println!("{}", self.buffer);
        self.autoflush()
    }

    fn write_few(&mut self, bits: u32, n_bits: usize) -> std::io::Result<usize> {
        self.buffer
            .extend_from_bitslice(&BitSlice::<u32, Msb0>::from_element(&bits)[(32 - n_bits)..]);
        //println!("{}", self.buffer);
        self.autoflush()
    }

    fn end_flush(&mut self) -> std::io::Result<usize> {
        let missing = (8 - self.buffer.len() % 8) % 8;
        self.buffer
            .extend_from_bitslice(&BitSlice::<usize, Msb0>::from_element(&0)[..missing]);
        let wrote = self.flush()?;
        assert_eq!(self.buffer.len(), 0);
        Ok(wrote)
    }
}
