use crate::{
    MessageType,
    decoders::DecodingBuffer,
    types::{Flags, Header},
};
use anyhow::{Result, ensure};

pub(crate) struct HeaderDecoder;

impl HeaderDecoder {
    pub(crate) fn decode(mut buffer: DecodingBuffer<'_>) -> Result<Header> {
        ensure!(buffer.len() >= Header::LENGTH);

        let _endian = buffer.skip();
        let message_type = MessageType::from(buffer.next_u8()?);
        let flags = Flags::try_from(buffer.next_u8()?)?;
        let _protocol_version = buffer.skip();
        let body_len = buffer.next_u32()? as usize;
        let serial = buffer.next_u32()?;
        let header_fields_len = buffer.next_u32()? as usize;

        Ok(Header {
            message_type,
            flags,
            body_len,
            serial,
            header_fields_len,
        })
    }
}
