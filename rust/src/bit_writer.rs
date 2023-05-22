use std::io::Write;

pub trait BitWrite {
    fn write_bit(&mut self, bit: bool) -> std::io::Result<usize>;
    fn write_byte(&mut self, byte: u8) -> std::io::Result<usize>;
    fn flush(&mut self) -> std::io::Result<usize>;
}

pub struct BitWriter<Writer: Write> {
    inner: Writer,
    /// Residual bits from last write
    residual: u8,
    /// Amount of residual bits
    residual_size: u8,
}

impl<Writer: Write> BitWriter<Writer> {
    pub fn new(writer: Writer) -> Self {
        Self {
            inner: writer,
            residual: 0,
            residual_size: 0,
        }
    }
}

impl<Writer: Write> BitWrite for BitWriter<Writer> {
    fn write_bit(&mut self, bit: bool) -> std::io::Result<usize> {
        self.residual |= (bit as u8) << (7 - self.residual_size);
        self.residual_size += 1;
        let mut written = 0;
        if self.residual_size == 8 {
            self.inner.write_all(&[self.residual])?;
            self.residual = 0;
            self.residual_size = 0;
            written = 1;
        }
        Ok(written)
    }

    fn write_byte(&mut self, byte: u8) -> std::io::Result<usize> {
        // byte: BBBBbbbb
        let previous = self.residual;
        // previous: rrrr____
        let new = byte >> self.residual_size;
        // new: ____BBBB
        let write_byte = previous | new;
        // write_byte: rrrrBBBB
        self.inner.write_all(&[write_byte])?;
        self.residual = byte
            .checked_shl((8 - self.residual_size) as u32)
            .unwrap_or(0);
        // residual: bbbb____
        Ok(1)
    }

    fn flush(&mut self) -> std::io::Result<usize> {
        let mut written = 0;
        if self.residual_size > 0 {
            let write_byte = self.residual;
            self.inner.write(&[write_byte])?;
            written = 1;
        }
        self.inner.flush()?;
        Ok(written)
    }
}
