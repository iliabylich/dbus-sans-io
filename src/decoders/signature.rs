use crate::types::Signature;
use anyhow::{Context as _, Result, bail, ensure};

pub(crate) struct SignatureDecoder;

impl SignatureDecoder {
    pub fn parse(s: &[u8]) -> Result<(Signature, usize)> {
        match s.get(0).copied().context("EOF")? {
            b'y' => Ok((Signature::Byte, 1)),
            b'b' => Ok((Signature::Bool, 1)),
            b'n' => Ok((Signature::Int16, 1)),
            b'q' => Ok((Signature::UInt16, 1)),
            b'i' => Ok((Signature::Int32, 1)),
            b'u' => Ok((Signature::UInt32, 1)),
            b'x' => Ok((Signature::Int64, 1)),
            b't' => Ok((Signature::UInt64, 1)),
            b'd' => Ok((Signature::Double, 1)),
            b'h' => Ok((Signature::UnixFD, 1)),

            b's' => Ok((Signature::String, 1)),
            b'o' => Ok((Signature::ObjectPath, 1)),
            b'g' => Ok((Signature::Signature, 1)),

            b'(' => {
                let mut fields = vec![];
                let mut offset = 1;
                while let Some(rem) = s.get(offset..) {
                    if let Ok((field, field_len)) = Self::parse(rem) {
                        fields.push(field);
                        offset += field_len
                    } else {
                        break;
                    }
                }
                ensure!(s.get(offset).copied() == Some(b')'));
                Ok((Signature::Struct(fields), offset + 1))
            }

            b'a' => {
                let (item, item_len) = Self::parse(&s[1..])?;
                Ok((Signature::Array(Box::new(item)), item_len + 1))
            }

            b'v' => Ok((Signature::Variant, 1)),

            other => bail!("unknown signature member: {}", other as char),
        }
    }

    pub fn parse_one_to_end(s: &[u8]) -> Result<Signature> {
        let (sig, len) = Self::parse(s)?;
        ensure!(len == s.len());
        Ok(sig)
    }

    pub fn parse_multi_to_end(mut s: &[u8]) -> Result<Vec<Signature>> {
        let mut out = vec![];
        while !s.is_empty() {
            let (sig, len) = Self::parse(s)?;
            s = &s[len..];
            out.push(sig);
        }
        Ok(out)
    }
}

#[test]
fn test_signature_decode() {
    assert_eq!(
        SignatureDecoder::parse_one_to_end(b"(isad(gh))").unwrap(),
        Signature::Struct(vec![
            Signature::Int32,
            Signature::String,
            Signature::Array(Box::new(Signature::Double)),
            Signature::Struct(vec![Signature::Signature, Signature::UnixFD])
        ])
    );
}
