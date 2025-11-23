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
    Struct(Vec<Signature>),
    Array(Box<Signature>),
    Variant,
}
