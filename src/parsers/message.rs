use anyhow::{Context as _, Result};

#[derive(Debug)]
pub struct MessageParser {
    pub data: Vec<u8>,
    pub pos: usize,
}

impl MessageParser {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        let byte = self.data.get(self.pos).copied().context("EOF")?;
        self.pos += 1;
        Ok(byte)
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        let value = u32::from_le_bytes([
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?,
        ]);
        Ok(value)
    }

    pub fn read_str(&mut self) -> Result<&str> {
        let len = self.read_u32()? as usize;
        let s = std::str::from_utf8(&self.data[self.pos..self.pos + len])
            .expect("invalid UTF-8 in string");
        self.pos += len + 1; // +1 for null terminator
        Ok(s)
    }
}
