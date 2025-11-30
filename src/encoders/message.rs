use crate::{
    encoders::{EncodingBuffer, HeaderEncoder, SignatureEncoder, ValueEncoder},
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
            if let Some(interface) = message.interface.as_ref() {
                buf.align(8);
                ValueEncoder::encode_header(
                    &mut buf,
                    HeaderFieldName::Interface,
                    &Value::String(interface.clone()),
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
            if let Some(error_name) = message.error_name.as_ref() {
                buf.align(8);
                ValueEncoder::encode_header(
                    &mut buf,
                    HeaderFieldName::ErrorName,
                    &Value::String(error_name.clone()),
                );
            }
            if let Some(reply_serial) = message.reply_serial.as_ref() {
                buf.align(8);
                ValueEncoder::encode_header(
                    &mut buf,
                    HeaderFieldName::ReplySerial,
                    &Value::UInt32(*reply_serial),
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
            if let Some(signature) = message.signature.as_ref() {
                buf.align(8);
                let mut sig_buf = EncodingBuffer::new();
                SignatureEncoder::encode_signature(&mut sig_buf, signature);
                let sig_buf = sig_buf.done();
                ValueEncoder::encode_header(
                    &mut buf,
                    HeaderFieldName::Signature,
                    &Value::Signature(sig_buf),
                );
            }
            if let Some(unix_fds) = message.unix_fds.as_ref() {
                buf.align(8);
                ValueEncoder::encode_header(
                    &mut buf,
                    HeaderFieldName::UnixFds,
                    &Value::UInt32(*unix_fds),
                );
            }
        };
        let header_fieldss_end = buf.size();

        buf.set_u32(12, (header_fieldss_end - header_fields_start) as u32)?;
        buf.align(8);

        // TODO: write body once we have some
        let body_starts_at = buf.size();
        for value in &message.body {
            ValueEncoder::encode_value(&mut buf, value);
        }
        let body_len = buf.size() - body_starts_at;
        buf.set_u32(4, body_len as u32)?;

        let buf = buf.done();

        Ok(buf)
    }
}
