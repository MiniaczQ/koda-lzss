use std::io::Write;

pub trait BitWrite {
    fn write_bit(&mut self, bit: bool) -> std::io::Result<()>;
    fn write_byte(&mut self, byte: u8) -> std::io::Result<()>;
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
    fn write_bit(&mut self, bit: bool) -> std::io::Result<()> {
        self.residual <<= 1;
        self.residual |= bit as u8;
        self.residual_size += 1;
        if self.residual_size == 8 {
            self.inner.write_all(&[self.residual])?;
            self.residual_size = 0;
        }
        Ok(())
    }

    fn write_byte(&mut self, byte: u8) -> std::io::Result<()> {
        let write_byte = self.residual | (byte << self.residual_size);
        self.inner.write_all(&[write_byte])?;
        self.residual = byte >> self.residual_size;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn check() {
        println!("{:?}", (0b00010101u8 << 1) | (true as u8));
    }
}
