use crate::MessageType;

#[derive(Default)]
pub(crate) struct Header<'a>(&'a [u8]);

impl<'a> Header<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self(bytes)
    }

    pub(crate) fn message_type(&self) -> MessageType {
        MessageType::from(self.0[1])
    }

    pub(crate) fn flags(&self) -> u8 {
        self.0[2]
    }

    pub(crate) fn body_len(&self) -> usize {
        u32::from_le_bytes([self.0[4], self.0[5], self.0[6], self.0[7]]) as usize
    }

    pub(crate) fn serial(&self) -> u32 {
        u32::from_le_bytes([self.0[8], self.0[9], self.0[10], self.0[11]])
    }

    pub(crate) fn header_fields_len(&self) -> usize {
        u32::from_le_bytes([self.0[12], self.0[13], self.0[14], self.0[15]]) as usize
    }

    pub(crate) fn padding_len(&self) -> usize {
        let read_so_far = 16 + self.header_fields_len();
        read_so_far.next_multiple_of(8) - read_so_far
    }

    pub(crate) fn has_header_fields(&self) -> bool {
        self.header_fields_len() > 0
    }

    pub(crate) fn has_padding(&self) -> bool {
        self.padding_len() > 0
    }

    pub(crate) fn has_body(&self) -> bool {
        self.body_len() > 0
    }
}

impl std::fmt::Debug for Header<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Header")
            .field("message_type", &self.message_type())
            .field("flags", &self.flags())
            .field("body_len", &self.body_len())
            .field("serial", &self.serial())
            .field("header_fields_len", &self.header_fields_len())
            .field("padding_len", &self.padding_len())
            .finish()
    }
}
