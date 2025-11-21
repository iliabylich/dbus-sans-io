use crate::{DynamicSizeReader, IoOperation, IoReader, MessageParser};
use anyhow::Result;
use std::io::Read;

pub(crate) struct BodyReader {
    len: usize,
    reader: DynamicSizeReader,
}

impl BodyReader {
    pub(crate) fn new(len: usize) -> Self {
        Self {
            len,
            reader: DynamicSizeReader::new(len),
        }
    }
}

impl IoReader<MessageParser> for BodyReader {
    fn continue_reading(&mut self, r: &mut impl Read) -> Result<IoOperation<MessageParser>> {
        if self.len == 0 {
            return Ok(IoOperation::Finished(MessageParser::new(vec![])));
        }

        match self.reader.continue_reading(r)? {
            IoOperation::Finished(bytes) => Ok(IoOperation::Finished(MessageParser::new(bytes))),
            IoOperation::WouldBlock => Ok(IoOperation::WouldBlock),
        }
    }
}
