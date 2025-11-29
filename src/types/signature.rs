#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CompleteType {
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

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Signature {
    pub(crate) items: Vec<CompleteType>,
}

impl std::fmt::Debug for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Signature(")?;
        let mut started = false;
        for item in &self.items {
            write!(f, "{}{:?}", if started { " -> " } else { "" }, item)?;
            started = true;
        }
        write!(f, ")")
    }
}
