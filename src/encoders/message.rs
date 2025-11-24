use crate::{
    encoders::{EncodingBuffer, HeaderEncoder, HeaderFieldsEncoder},
    types::Message,
};
use anyhow::Result;

pub(crate) struct MessageEncoder;

impl MessageEncoder {
    pub(crate) fn encode(message: &Message) -> Result<Vec<u8>> {
        let mut buf = EncodingBuffer::new();

        HeaderEncoder::encode(&mut buf, message.message_type, message.flags);

        HeaderFieldsEncoder::encode(
            &mut buf,
            message.path.as_ref(),
            message.interface.as_ref(),
            message.member.as_ref(),
            message.error_name.as_ref(),
            message.reply_serial.clone(),
            message.destination.as_ref(),
            message.sender.as_ref(),
            &message.body_signature,
            message.unix_fds,
        )?;

        // TODO: write body once we have some
        assert_eq!(message.body.len(), 0);
        let body_len = buf.size() - HeaderEncoder::HEADER_LEN;
        buf.align(8);

        HeaderEncoder::encode_body_len(&mut buf, body_len as u32)?;
        HeaderEncoder::encode_serial(&mut buf, message.serial)?;

        Ok(buf.done())
    }
}
