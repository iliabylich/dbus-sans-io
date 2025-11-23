use crate::{MessageType, decoders::DecodingBuffer, types::Flags};
use anyhow::{Result, ensure};

#[derive(Clone, Copy)]
pub(crate) struct Header {
    pub(crate) message_type: MessageType,
    pub(crate) flags: Flags,
    pub(crate) body_len: usize,
    pub(crate) serial: u32,
    pub(crate) header_fields_len: usize,
}

impl Header {
    pub(crate) fn padding_len(&self) -> usize {
        let read_so_far = HeaderDecoder::LENGTH + self.header_fields_len;
        read_so_far.next_multiple_of(8) - read_so_far
    }

    pub(crate) fn full_message_size(&self) -> usize {
        let mut out = HeaderDecoder::LENGTH + self.header_fields_len;
        out = out.next_multiple_of(8);
        out += self.body_len;
        out
    }
}

impl std::fmt::Debug for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Header")
            .field("message_type", &self.message_type)
            .field("flags", &self.flags)
            .field("body_len", &self.body_len)
            .field("serial", &self.serial)
            .field("header_fields_len", &self.header_fields_len)
            .field("padding_len", &self.padding_len())
            .finish()
    }
}

pub(crate) struct HeaderDecoder;

impl HeaderDecoder {
    pub(crate) const LENGTH: usize = 16;

    pub(crate) fn decode(mut buffer: DecodingBuffer<'_>) -> Result<Header> {
        ensure!(buffer.len() >= Self::LENGTH);

        let _ = buffer.skip()?;
        let message_type = MessageType::from(buffer.next_u8()?);
        let flags = Flags::try_from(buffer.next_u8()?)?;
        let _ = buffer.skip()?;
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
