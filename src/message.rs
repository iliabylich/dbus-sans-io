use crate::{
    MessageType,
    decoders::{Header, HeaderFields, Signature, Value},
};
use anyhow::Result;

#[derive(Debug, Default)]
pub struct Message {
    pub message_type: MessageType,
    pub flags: u8,
    pub serial: u32,

    pub member: Option<String>,
    pub interface: Option<String>,
    pub path: Option<Vec<u8>>,
    pub error_name: Option<String>,
    pub reply_serial: Option<u32>,
    pub destination: Option<String>,
    pub sender: Option<String>,
    pub signature: Vec<Signature>,
    pub unix_fds: Option<u32>,

    pub body: Vec<Value>,
}

impl Message {
    pub(crate) fn split(bytes: Vec<u8>) -> Result<Self> {
        let header = Header::new(&bytes)?;
        let message_type = header.message_type();
        let flags = header.flags();
        let serial = header.serial();
        let header_fields_len = header.header_fields_len();
        let padding_len = header.padding_len();

        let HeaderFields {
            member,
            interface,
            path,
            error_name,
            reply_serial,
            destination,
            sender,
            signature,
            unix_fds,
        } = HeaderFields::new(&bytes[..Header::LENGTH + header_fields_len], Header::LENGTH)?;

        let (signature, body) = match signature {
            Some(signature) => {
                let signatures = Signature::parse_multi(signature.as_bytes())?;
                let body_offset = Header::LENGTH + header_fields_len + padding_len;
                let (body, body_len) = Value::read_multi(&bytes, body_offset, &signatures)?;
                assert_eq!(body_len, header.body_len());
                (signatures, body)
            }
            None => (vec![], vec![]),
        };

        Ok(Self {
            message_type,
            flags,
            serial,

            member,
            interface,
            path,
            error_name,
            reply_serial,
            destination,
            sender,
            signature,
            unix_fds,

            body,
        })
    }
}
