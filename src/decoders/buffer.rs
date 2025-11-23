use anyhow::{Context, Result, ensure};

pub(crate) struct DecodingBuffer<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> DecodingBuffer<'a> {
    pub(crate) fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub(crate) fn set_pos(&mut self, pos: usize) {
        self.pos = pos;
    }

    pub(crate) fn with_pos(mut self, pos: usize) -> Self {
        self.set_pos(pos);
        self
    }

    pub(crate) fn len(&self) -> usize {
        self.data.len() - self.pos
    }

    pub(crate) fn next_u8(&mut self) -> Result<u8> {
        let byte = self.data.get(self.pos).context("EOF")?;
        self.pos += 1;
        Ok(*byte)
    }

    pub(crate) fn next_u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes([
            self.next_u8()?,
            self.next_u8()?,
            self.next_u8()?,
            self.next_u8()?,
        ]))
    }

    pub(crate) fn next_n(&mut self, count: usize) -> Result<&[u8]> {
        let bytes = self.data.get(self.pos..self.pos + count).context("EOF")?;
        self.pos += count;
        Ok(bytes)
    }

    pub(crate) fn is_eof(&self) -> bool {
        self.pos >= self.data.len()
    }

    pub(crate) fn skip_n(&mut self, count: usize) -> Result<()> {
        self.pos += count;
        ensure!(!self.is_eof());
        Ok(())
    }

    pub(crate) fn skip(&mut self) -> Result<()> {
        self.skip_n(1)
    }

    pub(crate) fn align(&mut self, align: usize) -> Result<()> {
        self.set_pos(self.pos.next_multiple_of(align));
        ensure!(!self.is_eof());
        Ok(())
    }
}
