use crate::types::{ObjectPath, signature::CompleteType};

#[derive(Debug)]
pub(crate) enum Value {
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
    pub(crate) fn complete_type(&self) -> CompleteType {
        ValueRef::from(self).complete_type()
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
            Value::ObjectPath(v) => Self::ObjectPath(v.as_bytes()),
            Value::Signature(v) => Self::Signature(v),
            Value::Struct(v) => Self::Struct(v),
            Value::Array(v) => Self::Array(v),
            Value::Variant(v) => Self::Variant(v),
        }
    }
}

impl ValueRef<'_> {
    pub(crate) fn complete_type(self) -> CompleteType {
        match self {
            Self::Byte(_) => CompleteType::Byte,
            Self::Bool(_) => CompleteType::Bool,
            Self::Int16(_) => CompleteType::Int16,
            Self::UInt16(_) => CompleteType::UInt16,
            Self::Int32(_) => CompleteType::Int32,
            Self::UInt32(_) => CompleteType::UInt32,
            Self::Int64(_) => CompleteType::Int64,
            Self::UInt64(_) => CompleteType::UInt64,
            Self::Double(_) => CompleteType::Double,
            Self::UnixFD(_) => CompleteType::UnixFD,
            Self::String(_) => CompleteType::String,
            Self::ObjectPath(_) => CompleteType::ObjectPath,
            Self::Signature(_) => CompleteType::Signature,
            Self::Struct(values) => {
                let mut types = vec![];
                for value in values {
                    types.push(ValueRef::from(value).complete_type());
                }
                CompleteType::Struct(types)
            }
            Self::Array(items) => {
                let Some(item) = items.first() else {
                    panic!("can't represent empty array")
                };
                let item_type = ValueRef::from(item).complete_type();
                for item in items {
                    if ValueRef::from(item).complete_type() != item_type {
                        panic!("heterogenous array")
                    }
                }
                CompleteType::Array(Box::new(item_type))
            }
            Self::Variant(_value) => CompleteType::Variant,
        }
    }
}
