use crate::{decoders::DecodingBuffer, types::Signature};
use anyhow::{Context as _, Result, bail, ensure};

pub(crate) struct SignatureDecoder;

impl SignatureDecoder {
    pub fn parse_one(buf: &mut DecodingBuffer) -> Result<Signature> {
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
                    let field = Self::parse_one(buf)?;
                    fields.push(field);
                }
                ensure!(buf.next_u8().is_ok_and(|b| b == b')'));
                Ok(Signature::Struct(fields))
            }

            b'a' => {
                let item = Self::parse_one(buf)?;
                Ok(Signature::Array(Box::new(item)))
            }

            b'v' => Ok(Signature::Variant),

            other => bail!("unknown signature member: {}", other as char),
        }
    }

    pub fn parse_multi(buf: &mut DecodingBuffer) -> Result<Vec<Signature>> {
        let mut out = vec![];
        while !buf.is_eof() {
            let sig = Self::parse_one(buf)?;
            out.push(sig);
        }
        Ok(out)
    }
}

#[test]
fn test_signature_decode() {
    let mut buf = DecodingBuffer::new(b"(isad(gh))");

    assert_eq!(
        SignatureDecoder::parse_one(&mut buf).unwrap(),
        Signature::Struct(vec![
            Signature::Int32,
            Signature::String,
            Signature::Array(Box::new(Signature::Double)),
            Signature::Struct(vec![Signature::Signature, Signature::UnixFD])
        ])
    );
}
