use crate::{IoOperation, IoReader};
use anyhow::{Result, bail};
use std::io::{ErrorKind, Read};

pub(crate) struct DynamicSizeReader {
    size: usize,
    buf: Vec<u8>,
    pos: usize,
    drained: bool,
}

impl DynamicSizeReader {
    pub(crate) fn new(size: usize) -> Self {
        Self {
            size,
            buf: vec![0; size],
            pos: 0,
            drained: false,
        }
    }
}

impl IoReader<Vec<u8>> for DynamicSizeReader {
    fn continue_reading(&mut self, r: &mut impl Read) -> Result<IoOperation<Vec<u8>>> {
        if self.drained {
            bail!("reader has already completed and been drained")
        }

        loop {
            if self.pos == self.size {
                break;
            }

            match r.read(&mut self.buf[self.pos..]) {
                Ok(len) => {
                    self.pos += len;
                }
                Err(err) if err.kind() == ErrorKind::WouldBlock => {
                    return Ok(IoOperation::WouldBlock);
                }
                Err(err) => return Err(err.into()),
            }
        }

        self.drained = true;
        Ok(IoOperation::Finished(std::mem::take(&mut self.buf)))
    }
}
