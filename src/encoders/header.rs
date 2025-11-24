use anyhow::Result;

use crate::{
    encoders::EncodingBuffer,
    types::{Flags, MessageType},
};

pub(crate) struct HeaderEncoder;

impl HeaderEncoder {
    pub(crate) const HEADER_LEN: usize = 16;
    const LITTLE_ENDIAN: u8 = b'l';
    const PROTOCOL_VERSION: u8 = 1;

    pub(crate) fn encode(buf: &mut EncodingBuffer, message_type: MessageType, flags: Flags) {
        buf.encode_u8(Self::LITTLE_ENDIAN);
        buf.encode_u8(message_type as u8);
        buf.encode_u8(flags.into());
        buf.encode_u8(Self::PROTOCOL_VERSION);
        buf.encode_u32(0);
        buf.encode_u32(0);
        buf.encode_u32(0);
    }

    pub(crate) fn encode_serial(buf: &mut EncodingBuffer, serial: u32) -> Result<()> {
        buf.set_u32(8, serial)
    }

    pub(crate) fn encode_body_len(buf: &mut EncodingBuffer, body_len: u32) -> Result<()> {
        buf.set_u32(12, body_len)
    }
}
