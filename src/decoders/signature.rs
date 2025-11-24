use crate::{
    decoders::DecodingBuffer,
    types::{MessageSignature, Signature},
};
use anyhow::{Result, bail, ensure};

pub(crate) struct SignatureDecoder;

impl SignatureDecoder {
    pub fn parse(buf: &mut DecodingBuffer) -> Result<Signature> {
        match buf.next_u8()? {
            b'y' => Ok(Signature::Byte),
            b'b' => Ok(Signature::Bool),
            b'n' => Ok(Signature::Int16),
            b'q' => Ok(Signature::UInt16),
            b'i' => Ok(Signature::Int32),
            b'u' => Ok(Signature::UInt32),
            b'x' => Ok(Signature::Int64),
            b't' => Ok(Signature::UInt64),
            b'd' => Ok(Signature::Double),
            b'h' => Ok(Signature::UnixFD),

            b's' => Ok(Signature::String),
            b'o' => Ok(Signature::ObjectPath),
            b'g' => Ok(Signature::Signature),

            b'(' => {
                let mut fields = vec![];
                while buf.peek().is_some_and(|b| b != b')') {
                    let field = Self::parse(buf)?;
                    fields.push(field);
                }
                ensure!(buf.next_u8().is_ok_and(|b| b == b')'));
                Ok(Signature::Struct(fields))
            }

            b'a' => {
                let item = Self::parse(buf)?;
                Ok(Signature::Array(Box::new(item)))
            }

            b'v' => {
                todo!()
            }

            other => bail!("unknown signature member: {}", other as char),
        }
    }

    pub fn parse_message_signature(buf: &mut DecodingBuffer) -> Result<MessageSignature> {
        let mut out = vec![];
        while !buf.is_eof() {
            let sig = Self::parse(buf)?;
            out.push(sig);
        }
        Ok(MessageSignature(out))
    }
}

#[test]
fn test_signature_decode() {
    let mut buf = DecodingBuffer::new(b"(isad(gh))");

    assert_eq!(
        SignatureDecoder::parse(&mut buf).unwrap(),
        Signature::Struct(vec![
            Signature::Int32,
            Signature::String,
            Signature::Array(Box::new(Signature::Double)),
            Signature::Struct(vec![Signature::Signature, Signature::UnixFD])
        ])
    );
}
