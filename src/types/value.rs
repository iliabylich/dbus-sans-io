use crate::types::{ObjectPath, Signature};
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
    ObjectPath(ObjectPath),
    Signature(Vec<u8>),
    Struct(Vec<Value>),
    Array(Vec<Value>),
    Variant(Box<Value>),
}

impl Value {
    pub fn signature(&self) -> Result<Signature> {
        ValueRef::from(self).signature()
    }
}

#[derive(Clone, Copy)]
pub(crate) enum ValueRef<'a> {
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

    String(&'a str),
    ObjectPath(&'a [u8]),
    Signature(&'a [u8]),
    Struct(&'a [Value]),
    Array(&'a [Value]),
    Variant(&'a Value),
}

impl<'a> From<&'a Value> for ValueRef<'a> {
    fn from(value: &'a Value) -> Self {
        match value {
            Value::Byte(v) => Self::Byte(*v),
            Value::Bool(v) => Self::Bool(*v),
            Value::Int16(v) => Self::Int16(*v),
            Value::UInt16(v) => Self::UInt16(*v),
            Value::Int32(v) => Self::Int32(*v),
            Value::UInt32(v) => Self::UInt32(*v),
            Value::Int64(v) => Self::Int64(*v),
            Value::UInt64(v) => Self::UInt64(*v),
            Value::Double(v) => Self::Double(*v),
            Value::UnixFD(v) => Self::UnixFD(*v),
            Value::String(v) => Self::String(v),
            Value::ObjectPath(v) => Self::ObjectPath(&v.0),
            Value::Signature(v) => Self::Signature(v),
            Value::Struct(v) => Self::Struct(v),
            Value::Array(v) => Self::Array(v),
            Value::Variant(v) => Self::Variant(v),
        }
    }
}

impl ValueRef<'_> {
    pub(crate) fn signature(self) -> Result<Signature> {
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
                    signatures.push(ValueRef::from(value).signature()?);
                }
                Signature::Struct(signatures)
            }
            Self::Array(items) => {
                let mut item_signature = Signature::Byte;
                if let Some(item) = items.first() {
                    item_signature = ValueRef::from(item).signature()?;
                }
                for item in items {
                    if ValueRef::from(item).signature()? != item_signature {
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
