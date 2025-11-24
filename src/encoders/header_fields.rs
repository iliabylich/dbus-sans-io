use crate::{
    encoders::{EncodingBuffer, SignatureEncoder, ValueEncoder},
    types::{HeaderField, MessageSignature, ObjectPath, ValueRef},
};
use anyhow::Result;

pub(crate) struct HeaderFieldsEncoder;

impl HeaderFieldsEncoder {
    pub(crate) fn encode(
        buf: &mut EncodingBuffer,
        path: Option<&ObjectPath>,
        interface: Option<&String>,
        member: Option<&String>,
        error_name: Option<&String>,
        reply_serial: Option<u32>,
        destination: Option<&String>,
        sender: Option<&String>,
        signature: &MessageSignature,
        unix_fds: Option<u32>,
    ) -> Result<()> {
        fn encode_header(
            buf: &mut EncodingBuffer,
            header_field: HeaderField,
            value: ValueRef<'_>,
        ) -> Result<()> {
            buf.align(8);
            buf.encode_u8(header_field.into());

            let sig = {
                let signature = value.signature()?;
                let mut buf = EncodingBuffer::new();
                SignatureEncoder::encode(&mut buf, &signature);
                buf.done()
            };
            ValueEncoder::encode_signature(buf, &sig);

            ValueEncoder::encode_value(buf, value);
            Ok(())
        }

        if let Some(path) = path {
            encode_header(buf, HeaderField::Path, ValueRef::ObjectPath(&path.0))?;
        }

        if let Some(interface) = interface {
            encode_header(buf, HeaderField::Interface, ValueRef::String(interface))?;
        }

        if let Some(member) = member {
            encode_header(buf, HeaderField::Member, ValueRef::String(member))?;
        }

        if let Some(error_name) = error_name {
            encode_header(buf, HeaderField::ErrorName, ValueRef::String(error_name))?;
        }

        if let Some(reply_serial) = reply_serial {
            encode_header(
                buf,
                HeaderField::ReplySerial,
                ValueRef::UInt32(reply_serial),
            )?;
        }

        if let Some(destination) = destination {
            encode_header(buf, HeaderField::Destination, ValueRef::String(destination))?;
        }

        if let Some(sender) = sender {
            encode_header(buf, HeaderField::Sender, ValueRef::String(sender))?;
        }

        if !signature.0.is_empty() {
            let signature = {
                let mut buf = EncodingBuffer::new();
                for sig in &signature.0 {
                    SignatureEncoder::encode(&mut buf, sig);
                }
                buf.done()
            };
            encode_header(buf, HeaderField::Signature, ValueRef::Signature(&signature))?;
        }

        if let Some(unix_fds) = unix_fds {
            encode_header(buf, HeaderField::UnixFds, ValueRef::UInt32(unix_fds))?;
        }

        Ok(())
    }
}
