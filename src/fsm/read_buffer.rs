#[derive(Debug, Default)]
pub(crate) struct ReadBuffer {
    buf: Vec<u8>,
    pos: usize,
}

impl ReadBuffer {
    pub(crate) fn new(size: usize) -> Self {
        Self {
            buf: vec![0; size],
            pos: 0,
        }
    }

    pub(crate) fn resize(&mut self, new_size: usize) {
        let len = self.buf.len();
        self.buf.reserve_exact(new_size - len);
        while self.buf.len() != new_size {
            self.buf.push(0)
        }
    }

    pub(crate) fn filled_part(&self) -> &[u8] {
        &self.buf[..self.pos]
    }

    pub(crate) fn remaining_part(&mut self) -> &mut [u8] {
        &mut self.buf[self.pos..]
    }

    pub(crate) fn is_full(&self) -> bool {
        self.pos == self.buf.len()
    }

    pub(crate) fn into_vec(self) -> Vec<u8> {
        assert!(self.is_full());
        self.buf
    }

    pub(crate) fn add_pos(&mut self, len: usize) {
        self.pos += len;
        assert!(self.pos <= self.buf.len())
    }

    pub(crate) fn take(&mut self) -> Self {
        std::mem::take(self)
    }
}
