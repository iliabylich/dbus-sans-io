use crate::MessageType;
use anyhow::Result;

#[derive(Default, Debug)]
pub(crate) struct Header {
    pub(crate) message_type: MessageType,
    pub(crate) flags: u8,
    pub(crate) body_len: usize,
    pub(crate) serial: u32,
    pub(crate) header_fields_len: usize,
}

impl Header {
    pub(crate) fn padding_len(&self) -> usize {
        let read_so_far = 16 + self.header_fields_len;
        read_so_far.next_multiple_of(8) - read_so_far
    }

    pub(crate) fn has_header_fields(&self) -> bool {
        self.header_fields_len > 0
    }

    pub(crate) fn has_padding(&self) -> bool {
        self.padding_len() > 0
    }

    pub(crate) fn has_body(&self) -> bool {
        self.body_len > 0
    }
}

pub(crate) struct HeaderParser;

impl HeaderParser {
    pub(crate) fn parse(bytes: [u8; 16]) -> Result<Header> {
        let message_type = MessageType::try_from(bytes[1])?;
        let flags = bytes[2];
        let body_len = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let serial = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let header_fields_len = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
        Ok(Header {
            message_type,
            flags,
            body_len: body_len as usize,
            serial,
            header_fields_len: header_fields_len as usize,
        })
    }
}
