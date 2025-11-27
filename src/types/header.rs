use crate::types::{Flags, MessageType};

#[derive(Debug, Default)]
pub(crate) struct Header {
    pub(crate) message_type: MessageType,
    pub(crate) flags: Flags,
    pub(crate) body_len: usize,
    pub(crate) serial: u32,
}
