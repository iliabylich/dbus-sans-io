use crate::{
    MessageParser, MessageType,
    parsers::{Header, HeaderFields},
};

#[derive(Debug)]
pub struct Message {
    pub message_type: MessageType,
    pub flags: u8,
    pub serial: u32,
    pub member: Option<String>,
    pub interface: Option<String>,
    pub path: Option<String>,
    pub body: MessageParser,
}

impl Message {
    pub(crate) fn new(header: Header, header_fields: HeaderFields, body: Vec<u8>) -> Self {
        Self {
            message_type: header.message_type,
            flags: header.flags,
            serial: header.serial,
            member: header_fields.member,
            interface: header_fields.interface,
            path: header_fields.path,
            body: MessageParser::new(body),
        }
    }
}
