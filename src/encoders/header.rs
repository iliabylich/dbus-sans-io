use crate::{encoders::EncodingBuffer, types::Header};
use anyhow::Result;

pub(crate) struct HeaderEncoder;

impl HeaderEncoder {
    const LITTLE_ENDIAN: u8 = b'l';
    const PROTOCOL_VERSION: u8 = 1;

    pub(crate) fn encode_as_zeroes(buf: &mut EncodingBuffer) {
        buf.encode_u64(0);
        buf.encode_u64(0);
    }

    pub(crate) fn reencode(buf: &mut EncodingBuffer, header: Header) -> Result<()> {
        buf.set_u8(0, Self::LITTLE_ENDIAN)?;
        buf.set_u8(1, header.message_type as u8)?;
        buf.set_u8(2, header.flags.into())?;
        buf.set_u8(3, Self::PROTOCOL_VERSION)?;
        buf.set_u32(8, header.serial)?;
        buf.set_u32(12, header.body_len as u32)?;

        Ok(())
    }
}
