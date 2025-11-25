use crate::types::{Flags, MessageType};

pub struct Header {
    pub message_type: MessageType,
    pub flags: Flags,
    pub body_len: usize,
    pub serial: u32,
    pub header_fields_len: usize,
}

impl Header {
    pub const LENGTH: usize = 16;

    pub(crate) fn padding_len(&self) -> usize {
        let read_so_far = Self::LENGTH + self.header_fields_len;
        read_so_far.next_multiple_of(8) - read_so_far
    }

    pub(crate) fn full_message_size(&self) -> usize {
        let mut out = Self::LENGTH + self.header_fields_len;
        out = out.next_multiple_of(8);
        out += self.body_len;
        out
    }

    pub(crate) fn body_offset(&self) -> usize {
        (Self::LENGTH + self.header_fields_len).next_multiple_of(8)
    }
}
