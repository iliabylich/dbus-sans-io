use crate::{
    encoders::{EncodingBuffer, HeaderEncoder, ValueEncoder},
    types::{HeaderFieldName, Message, Value},
};
use anyhow::Result;

pub(crate) struct MessageEncoder;

impl MessageEncoder {
    pub(crate) fn encode(message: &Message) -> Result<Vec<u8>> {
        let mut buf = EncodingBuffer::new();

        HeaderEncoder::encode(&mut buf, &message.header)?;

        buf.encode_u32(0); // header fields len
        let header_fields_start = buf.size();
        {
            if let Some(path) = message.path.as_ref() {
                buf.align(8);
                ValueEncoder::encode_header(
                    &mut buf,
                    HeaderFieldName::Path,
                    &Value::ObjectPath(path.clone()),
                );
            }
            if let Some(member) = message.member.as_ref() {
                buf.align(8);
                ValueEncoder::encode_header(
                    &mut buf,
                    HeaderFieldName::Member,
                    &Value::String(member.clone()),
                );
            }
            if let Some(interface) = message.interface.as_ref() {
                buf.align(8);
                ValueEncoder::encode_header(
                    &mut buf,
                    HeaderFieldName::Interface,
                    &Value::String(interface.clone()),
                );
            }
            if let Some(destination) = message.destination.as_ref() {
                buf.align(8);
                ValueEncoder::encode_header(
                    &mut buf,
                    HeaderFieldName::Destination,
                    &Value::String(destination.clone()),
                );
            }
            if let Some(sender) = message.sender.as_ref() {
                buf.align(8);
                ValueEncoder::encode_header(
                    &mut buf,
                    HeaderFieldName::Sender,
                    &Value::String(sender.clone()),
                );
            }
        };
        let header_fieldss_end = buf.size();

        buf.set_u32(12, (header_fieldss_end - header_fields_start) as u32)?;
        buf.align(8);

        // TODO: write body once we have some
        let body_starts_at = buf.size();
        assert_eq!(message.body.len(), 0);
        let body_len = buf.size() - body_starts_at;
        buf.set_u32(4, body_len as u32)?;

        let buf = buf.done();

        Ok(buf)
    }
}
