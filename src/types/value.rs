use crate::types::Signature;
use anyhow::{Result, bail};

#[derive(Debug)]
pub enum Value {
    Byte(u8),
    Bool(bool),
    Int16(i16),
    UInt16(u16),
    Int32(i32),
    UInt32(u32),
    Int64(i64),
    UInt64(u64),
    Double(f64),
    UnixFD(u32),

    String(String),
    ObjectPath(Vec<u8>),
    Signature(String),
    Struct(Vec<Value>),
    Array(Vec<Value>),
    Variant(Box<Value>),
}

impl Value {
    fn signature(&self) -> Result<Signature> {
        let sig = match self {
            Self::Byte(_) => Signature::Byte,
            Self::Bool(_) => Signature::Bool,
            Self::Int16(_) => Signature::Int16,
            Self::UInt16(_) => Signature::UInt16,
            Self::Int32(_) => Signature::Int32,
            Self::UInt32(_) => Signature::UInt32,
            Self::Int64(_) => Signature::Int64,
            Self::UInt64(_) => Signature::UInt64,
            Self::Double(_) => Signature::Double,
            Self::UnixFD(_) => Signature::UnixFD,
            Self::String(_) => Signature::String,
            Self::ObjectPath(_) => Signature::ObjectPath,
            Self::Signature(_) => Signature::Signature,
            Self::Struct(values) => {
                let mut signatures = vec![];
                for value in values {
                    signatures.push(value.signature()?);
                }
                Signature::Struct(signatures)
            }
            Self::Array(items) => {
                let mut item_signature = Signature::Byte;
                if let Some(item) = items.first() {
                    item_signature = item.signature()?;
                }
                for item in items {
                    if item.signature()? != item_signature {
                        bail!("heterogenous array")
                    }
                }
                Signature::Array(Box::new(item_signature))
            }
            Self::Variant(_value) => {
                todo!()
            }
        };
        Ok(sig)
    }
}
