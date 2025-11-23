use crate::{
    MessageParser, MessageType,
    parsers::{Header, HeaderFields},
};
use anyhow::Result;

#[derive(Debug, Default)]
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
    pub(crate) fn split(bytes: Vec<u8>) -> Result<Self> {
        let header = Header::new(&bytes)?;
        println!("{header:?}");
        let message_type = header.message_type();
        let flags = header.flags();
        let serial = header.serial();
        let header_fields_len = header.header_fields_len();
        let padding_len = header.padding_len();

        let HeaderFields {
            member,
            interface,
            path,
        } = HeaderFields::new(&bytes[16..]);

        let body_offset = 16 + header_fields_len + padding_len;
        let body = MessageParser::new(bytes, body_offset);

        Ok(Self {
            message_type,
            flags,
            serial,
            member,
            interface,
            path,
            body,
        })
    }
}
