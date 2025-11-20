use crate::{FixedSizeReader, IoOperation, IoReader, MessageType};
use anyhow::Result;
use std::io::Read;

pub(crate) struct HeaderReader(FixedSizeReader<16>);

impl HeaderReader {
    pub(crate) fn new() -> Self {
        Self(FixedSizeReader::new())
    }
}

impl IoReader<Header> for HeaderReader {
    fn continue_reading(&mut self, r: &mut impl Read) -> Result<IoOperation<Header>> {
        match self.0.continue_reading(r)? {
            IoOperation::Finished(bytes) => {
                let header = Header::try_from(bytes)?;
                Ok(IoOperation::Finished(header))
            }
            IoOperation::WouldBlock => Ok(IoOperation::WouldBlock),
        }
    }
}

pub(crate) struct Header {
    pub(crate) message_type: MessageType,
    pub(crate) flags: u8,
    pub(crate) body_len: u32,
    pub(crate) serial: u32,
    pub(crate) headers_len: u32,
}

impl TryFrom<[u8; 16]> for Header {
    type Error = anyhow::Error;

    fn try_from(data: [u8; 16]) -> Result<Self> {
        let message_type = MessageType::try_from(data[1])?;
        let flags = data[2];
        let body_len = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let serial = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        let headers_len = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
        Ok(Self {
            message_type,
            flags,
            body_len,
            serial,
            headers_len,
        })
    }
}
