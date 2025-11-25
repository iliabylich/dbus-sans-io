use crate::{
    MessageType,
    types::{Flags, MessageSignature, ObjectPath, Value},
};

#[derive(Debug, Default)]
pub struct Message {
    pub message_type: MessageType,
    pub flags: Flags,
    pub serial: u32,

    pub member: Option<String>,
    pub interface: Option<String>,
    pub path: Option<ObjectPath>,
    pub error_name: Option<String>,
    pub reply_serial: Option<u32>,
    pub destination: Option<String>,
    pub sender: Option<String>,
    pub signature: MessageSignature,
    pub unix_fds: Option<u32>,

    pub body: Vec<Value>,
}
