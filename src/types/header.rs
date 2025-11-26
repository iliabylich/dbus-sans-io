use crate::types::{Flags, MessageType};

#[derive(Debug, Default)]
pub struct Header {
    pub message_type: MessageType,
    pub flags: Flags,
    pub body_len: usize,
    pub serial: u32,
}
