use crate::{IoOperation, IoReader};
use anyhow::Result;
use std::io::{ErrorKind, Read};

pub(crate) struct FixedSizeReader<const N: usize> {
    buf: [u8; N],
    pos: usize,
}

impl<const N: usize> FixedSizeReader<N> {
    pub(crate) fn new() -> Self {
        Self {
            buf: [0; N],
            pos: 0,
        }
    }
}

impl<const N: usize> IoReader<[u8; N]> for FixedSizeReader<N> {
    fn continue_reading(&mut self, r: &mut impl Read) -> Result<IoOperation<[u8; N]>> {
        loop {
            if self.pos == N {
                break;
            }

            match r.read(&mut self.buf[self.pos..]) {
                Ok(len) => {
                    self.pos += len;
                }
                Err(err) if err.kind() == ErrorKind::WouldBlock => {
                    return Ok(IoOperation::WouldBlock);
                }
                Err(err) => {
                    return Err(err.into());
                }
            }
        }

        Ok(IoOperation::Finished(self.buf))
    }
}
