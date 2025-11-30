use crate::{
    Message,
    decoders::{DecodingBuffer, HeaderDecoder, ValueDecoder, signature::SignatureDecoder},
    types::{CompleteType, HeaderFieldName, Value},
};
use anyhow::{Result, bail, ensure};

pub(crate) struct MessageDecoder;

impl MessageDecoder {
    pub(crate) fn decode(bytes: &[u8]) -> Result<Message> {
        let mut buf = DecodingBuffer::new(bytes);
        let header = HeaderDecoder::decode(&mut buf)?;

        let mut path = None;
        let mut interface = None;
        let mut member = None;
        let mut error_name = None;
        let mut reply_serial = None;
        let mut destination = None;
        let mut sender = None;
        let mut signature = None;
        let mut unix_fds = None;

        let len = buf.next_u32()?;
        let end = buf.pos() + len as usize;
        let header_field_type =
            CompleteType::Struct(vec![CompleteType::Byte, CompleteType::Variant]);

        while buf.pos() < end {
            buf.align(8)?;
            let header_field =
                ValueDecoder::decode_value_by_complete_type(&mut buf, &header_field_type)?;

            let Value::Struct(pair) = header_field else {
                bail!("got {header_field:?} instead of a header field struct");
            };
            ensure!(pair.len() == 2);
            let mut pair = pair.into_iter();
            let header_field_name = pair.next().unwrap();
            let value = pair.next().unwrap();

            let Value::Byte(header_field_name) = header_field_name else {
                bail!("got {header_field_name:?} instead of a header field name");
            };
            let header_field_name = HeaderFieldName::from(header_field_name);

            let Value::Variant(value) = value else {
                bail!("got {value:?} instead of Variant in a header field");
            };

            match (header_field_name, *value) {
                (HeaderFieldName::Path, Value::ObjectPath(value)) => {
                    path = Some(value);
                }
                (HeaderFieldName::Interface, Value::String(value)) => {
                    interface = Some(value);
                }
                (HeaderFieldName::Member, Value::String(value)) => {
                    member = Some(value);
                }
                (HeaderFieldName::ErrorName, Value::String(value)) => {
                    error_name = Some(value);
                }
                (HeaderFieldName::ReplySerial, Value::UInt32(value)) => {
                    reply_serial = Some(value);
                }
                (HeaderFieldName::Destination, Value::String(value)) => {
                    destination = Some(value);
                }
                (HeaderFieldName::Sender, Value::String(value)) => {
                    sender = Some(value);
                }
                (HeaderFieldName::Signature, Value::Signature(value)) => {
                    let mut buf = DecodingBuffer::new(&value);
                    signature = Some(SignatureDecoder::decode_signature(&mut buf)?);
                }
                (HeaderFieldName::UnixFds, Value::UInt32(value)) => {
                    unix_fds = Some(value);
                }
                (header_field_name, value) => {
                    bail!(
                        "invalid combination of header field name/value: {header_field_name:?} vs {value:?}"
                    );
                }
            }
        }

        let mut body = vec![];
        if let Some(signature) = signature.as_ref() {
            buf.align(8)?;
            body = ValueDecoder::decode_values_by_signature(&mut buf, signature)?;
        }

        Ok(Message {
            header,
            body,
            path,
            interface,
            member,
            error_name,
            reply_serial,
            destination,
            sender,
            signature,
            unix_fds,
        })
    }
}
