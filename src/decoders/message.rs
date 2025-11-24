use crate::{
    Message,
    decoders::{
        DecodingBuffer, HeaderDecoder, HeaderFieldsDecoder, ValueDecoder,
        signature::SignatureDecoder,
    },
    types::ObjectPath,
};
use anyhow::{Result, ensure};

pub(crate) struct MessageDecoder;

impl MessageDecoder {
    pub(crate) fn decode(bytes: Vec<u8>) -> Result<Message> {
        let buffer = DecodingBuffer::new(&bytes[..HeaderDecoder::LENGTH]);
        let header = HeaderDecoder::decode(buffer)?;

        let message_type = header.message_type;
        let flags = header.flags;
        let serial = header.serial;
        let header_fields_len = header.header_fields_len;

        let buffer = DecodingBuffer::new(&bytes[..HeaderDecoder::LENGTH + header_fields_len])
            .with_pos(HeaderDecoder::LENGTH);

        let HeaderFieldsDecoder {
            member,
            interface,
            path,
            error_name,
            reply_serial,
            destination,
            sender,
            body_signature,
            unix_fds,
        } = HeaderFieldsDecoder::new(buffer)?;

        let path = path.map(ObjectPath::new);

        let (body_signature, body) = match body_signature {
            Some(signature) => {
                let mut signature_buf = DecodingBuffer::new(signature.as_bytes());
                let signatures = SignatureDecoder::parse_multi(&mut signature_buf)?;
                ensure!(signature_buf.is_eof());
                let mut buf = DecodingBuffer::new(&bytes).with_pos(header.body_offset());
                let body = ValueDecoder::read_multi(&mut buf, &signatures)?;
                assert!(buf.is_eof());
                (signatures, body)
            }
            None => (vec![], vec![]),
        };

        Ok(Message {
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
            body_signature,
            unix_fds,

            body,
        })
    }
}
