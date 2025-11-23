use crate::{
    Message,
    decoders::{HeaderDecoder, HeaderFieldsDecoder, ValueDecoder, signature::SignatureDecoder},
    types::{Flags, ObjectPath},
};
use anyhow::Result;

pub(crate) struct MessageDecoder;

impl MessageDecoder {
    pub(crate) fn decode(bytes: Vec<u8>) -> Result<Message> {
        let header = HeaderDecoder::new(&bytes)?;
        let message_type = header.message_type();
        let flags = Flags::try_from(header.flags())?;
        let serial = header.serial();
        let header_fields_len = header.header_fields_len();
        let padding_len = header.padding_len();

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
        } = HeaderFieldsDecoder::new(
            &bytes[..HeaderDecoder::LENGTH + header_fields_len],
            HeaderDecoder::LENGTH,
        )?;

        let path = path.map(ObjectPath::new);

        let (body_signature, body) = match body_signature {
            Some(signature) => {
                let signatures = SignatureDecoder::parse_multi_to_end(signature.as_bytes())?;
                let body_offset = HeaderDecoder::LENGTH + header_fields_len + padding_len;
                let (body, body_len) = ValueDecoder::read_multi(&bytes, body_offset, &signatures)?;
                assert_eq!(body_len, header.body_len());
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
