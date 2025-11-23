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

    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.pos]
    }

    pub(crate) fn remainder(&mut self) -> &mut [u8] {
        &mut self.buf[self.pos..]
    }

    pub(crate) fn is_full(&self) -> bool {
        self.pos == self.buf.len()
    }

    pub(crate) fn unwrap(self) -> Vec<u8> {
        assert!(self.is_full());
        self.buf
    }

    pub(crate) fn written(&mut self, len: usize) {
        self.pos += len;
        assert!(self.pos <= self.buf.len())
    }

    pub(crate) fn take(&mut self) -> Self {
        std::mem::take(self)
    }

    pub(crate) fn grow(&mut self, additional: usize) {
        let len = self.buf.len();
        self.buf.reserve_exact(additional);
        while self.buf.len() != len + additional {
            self.buf.push(0)
        }
    }
}
