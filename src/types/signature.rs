#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompleteType {
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
    Struct(Vec<CompleteType>),
    Array(Box<CompleteType>),
    Variant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    pub items: Vec<CompleteType>,
}
