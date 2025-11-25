use crate::{
    Message,
    decoders::{
        DecodingBuffer, HeaderDecoder, HeaderFieldsDecoder, ValueDecoder,
        signature::SignatureDecoder,
    },
    types::{Header, MessageSignature},
};
use anyhow::{Result, ensure};

pub(crate) struct MessageDecoder;

impl MessageDecoder {
    pub(crate) fn decode(bytes: Vec<u8>) -> Result<Message> {
        let buf = DecodingBuffer::new(&bytes[..Header::LENGTH]);
        let header = HeaderDecoder::decode(buf)?;

        let message_type = header.message_type;
        let flags = header.flags;
        let serial = header.serial;
        let header_fields_len = header.header_fields_len;

        let buffer = DecodingBuffer::new(&bytes[..Header::LENGTH + header_fields_len])
            .with_pos(Header::LENGTH);

        let HeaderFieldsDecoder {
            member,
            interface,
            path,
            error_name,
            reply_serial,
            destination,
            sender,
            signature,
            unix_fds,
        } = HeaderFieldsDecoder::new(buffer)?;

        let (signature, body) = match signature {
            Some(signature) => {
                let signature = {
                    let mut buf = DecodingBuffer::new(&signature);
                    let signature = SignatureDecoder::parse_message_signature(&mut buf)?;
                    ensure!(buf.is_eof());
                    signature
                };

                let body = {
                    let mut buf = DecodingBuffer::new(&bytes).with_pos(header.body_offset());
                    let body = ValueDecoder::decode_many(&mut buf, &signature.0)?;
                    assert!(buf.is_eof());
                    body
                };

                (signature, body)
            }
            None => (MessageSignature(vec![]), vec![]),
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
            signature,
            unix_fds,

            body,
        })
    }
}
