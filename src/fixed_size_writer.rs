use crate::{IoOperation, IoWriter};
use anyhow::Result;
use std::io::{ErrorKind, Write};

pub(crate) struct FixedSizeWriter<const N: usize> {
    buf: [u8; N],
    pos: usize,
}

impl<const N: usize> FixedSizeWriter<N> {
    pub(crate) fn new(buf: [u8; N]) -> Self {
        Self { buf, pos: 0 }
    }
}

impl<const N: usize> IoWriter<()> for FixedSizeWriter<N> {
    fn continue_writing(&mut self, w: &mut impl Write) -> Result<IoOperation<()>> {
        loop {
            if self.pos == N {
                break;
            }

            match w.write(&self.buf[self.pos..]) {
                Ok(len) => {
                    self.pos += len;
                }
                Err(err) if err.kind() == ErrorKind::WouldBlock => {
                    return Ok(IoOperation::WouldBlock);
                }
                Err(err) => return Err(err.into()),
            }
        }

        Ok(IoOperation::Finished(()))
    }
}
