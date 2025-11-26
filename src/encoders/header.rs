use crate::{encoders::EncodingBuffer, types::Header};
use anyhow::Result;

pub(crate) struct HeaderEncoder;

impl HeaderEncoder {
    const LITTLE_ENDIAN: u8 = b'l';
    const PROTOCOL_VERSION: u8 = 1;

    pub(crate) fn encode(buf: &mut EncodingBuffer, header: &Header) -> Result<()> {
        buf.encode_u8(Self::LITTLE_ENDIAN);
        buf.encode_u8(header.message_type as u8);
        buf.encode_u8(header.flags.into());
        buf.encode_u8(Self::PROTOCOL_VERSION);
        buf.encode_u32(0); // body len
        buf.encode_u32(header.serial);

        Ok(())
    }
}
