use std::io::{self, Write};

/// Bit writer (LSB-first per Zstd block coding; aligns to byte on flush as needed)
pub struct BitWriter<W: Write> {
    out: W,
    buf: u64,
    nbits: u8,
}

impl<W: Write> BitWriter<W> {
    pub fn new(out: W) -> Self { Self { out, buf: 0, nbits: 0 } }

    #[inline]
    pub fn write_bits(&mut self, bits: u64, n: u8) -> io::Result<()> {
        debug_assert!(n <= 56, "write_bits too large");
        self.buf |= (bits & ((1u64<<n)-1)) << self.nbits;
        self.nbits += n;
        while self.nbits >= 8 {
            let byte = (self.buf & 0xFF) as u8;
            self.out.write_all(&[byte])?;
            self.buf >>= 8;
            self.nbits -= 8;
        }
        Ok(())
    }

    pub fn align_to_byte(&mut self) -> io::Result<()> {
        if self.nbits > 0 {
            let byte = (self.buf & 0xFF) as u8;
            self.out.write_all(&[byte])?;
            self.buf >>= 8;
            self.nbits = 0;
        }
        Ok(())
    }

    pub fn into_inner(mut self) -> io::Result<W> {
        self.align_to_byte()?;
        Ok(self.out)
    }
}
