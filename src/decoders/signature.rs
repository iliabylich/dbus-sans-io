use anyhow::{Context as _, Result, bail, ensure};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Signature {
    Byte,
    Bool,
    Int16,
    UInt16,
    Int32,
    UInt32,
    Int64,
    UInt64,
    Double,
    UnixFD,

    String,
    ObjectPath,
    Signature,
    Struct(Vec<Signature>), // "(" and ")" are used instead
    Array(Box<Signature>),
    Variant,
}

impl Signature {
    pub fn parse(s: &[u8]) -> Result<(Self, usize)> {
        match s.get(0).copied().context("EOF")? {
            b'y' => Ok((Self::Byte, 1)),
            b'b' => Ok((Self::Bool, 1)),
            b'n' => Ok((Self::Int16, 1)),
            b'q' => Ok((Self::UInt16, 1)),
            b'i' => Ok((Self::Int32, 1)),
            b'u' => Ok((Self::UInt32, 1)),
            b'x' => Ok((Self::Int64, 1)),
            b't' => Ok((Self::UInt64, 1)),
            b'd' => Ok((Self::Double, 1)),
            b'h' => Ok((Self::UnixFD, 1)),

            b's' => Ok((Self::String, 1)),
            b'o' => Ok((Self::ObjectPath, 1)),
            b'g' => Ok((Self::Signature, 1)),

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
                Ok((Self::Struct(fields), offset + 1))
            }

            b'a' => {
                let (item, item_len) = Self::parse(&s[1..])?;
                Ok((Self::Array(Box::new(item)), item_len + 1))
            }

            b'v' => Ok((Self::Variant, 1)),

            other => bail!("unknown signature member: {}", other as char),
        }
    }

    pub fn parse_to_end(s: &[u8]) -> Result<Self> {
        let (sig, len) = Self::parse(s)?;
        ensure!(len == s.len());
        Ok(sig)
    }

    pub fn parse_multi(mut s: &[u8]) -> Result<Vec<Self>> {
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
fn test_signature_parse() {
    assert_eq!(
        Signature::parse_to_end(b"(isad(gh))").unwrap(),
        Signature::Struct(vec![
            Signature::Int32,
            Signature::String,
            Signature::Array(Box::new(Signature::Double)),
            Signature::Struct(vec![Signature::Signature, Signature::UnixFD])
        ])
    );
}
