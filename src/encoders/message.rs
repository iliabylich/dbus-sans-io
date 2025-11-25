use crate::{
    encoders::{EncodingBuffer, HeaderEncoder, HeaderFieldsEncoder},
    types::{Header, Message},
};
use anyhow::Result;

pub(crate) struct MessageEncoder;

impl MessageEncoder {
    pub(crate) fn encode(message: &Message) -> Result<Vec<u8>> {
        let mut buf = EncodingBuffer::new();

        HeaderEncoder::encode_as_zeroes(&mut buf);

        HeaderFieldsEncoder::encode(
            &mut buf,
            message.path.as_ref(),
            message.interface.as_ref(),
            message.member.as_ref(),
            message.error_name.as_ref(),
            message.reply_serial,
            message.destination.as_ref(),
            message.sender.as_ref(),
            &message.signature,
            message.unix_fds,
        )?;

        // TODO: write body once we have some
        assert_eq!(message.body.len(), 0);
        let body_len = buf.size() - Header::LENGTH;
        buf.align(8);

        HeaderEncoder::reencode(
            &mut buf,
            Header {
                message_type: message.message_type,
                flags: message.flags,
                body_len: body_len,
                serial: message.serial,
                header_fields_len: 0,
            },
        )?;

        Ok(buf.done())
    }
}
