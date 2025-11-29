use crate::types::{Flags, MessageType};

#[derive(Default)]
pub(crate) struct Header {
    pub(crate) message_type: MessageType,
    pub(crate) flags: Flags,
    pub(crate) body_len: usize,
    pub(crate) serial: u32,
}

impl std::fmt::Debug for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}({:?})", self.message_type, self.flags)
    }
}
