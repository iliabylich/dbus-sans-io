use crate::{encoders::EncodingBuffer, types::Signature};

pub(crate) struct SignatureEncoder;

impl SignatureEncoder {
    pub(crate) fn encode(buf: &mut EncodingBuffer, signature: &Signature) {
        match signature {
            Signature::Byte => buf.encode_u8(b'y'),
            Signature::Bool => buf.encode_u8(b'b'),
            Signature::Int16 => buf.encode_u8(b'n'),
            Signature::UInt16 => buf.encode_u8(b'q'),
            Signature::Int32 => buf.encode_u8(b'i'),
            Signature::UInt32 => buf.encode_u8(b'u'),
            Signature::Int64 => buf.encode_u8(b'x'),
            Signature::UInt64 => buf.encode_u8(b't'),
            Signature::Double => buf.encode_u8(b'd'),
            Signature::UnixFD => buf.encode_u8(b'h'),

            Signature::String => buf.encode_u8(b's'),
            Signature::ObjectPath => buf.encode_u8(b'o'),
            Signature::Signature => buf.encode_u8(b'g'),

            Signature::Struct(fields) => {
                buf.encode_u8(b'(');
                for field in fields {
                    Self::encode(buf, field);
                }
                buf.encode_u8(b')');
            }
            Signature::Array(item) => {
                buf.encode_u8(b'a');
                Self::encode(buf, &*item);
            }
            Signature::Variant => {
                buf.encode_u8(b'v');
                todo!()
            }
        }
    }
}
