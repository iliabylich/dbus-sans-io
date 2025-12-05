use anyhow::ensure;

#[derive(PartialEq, Eq, Default)]
pub(crate) struct Guid(pub(crate) Vec<u8>);

impl Guid {
    pub(crate) const LENGTH: usize = 37;

    pub(crate) fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.0[3..Self::LENGTH - 2])
    }
}

impl std::fmt::Debug for Guid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_str() {
            Ok(s) => write!(f, "{:?}", s),
            Err(err) => write!(f, "Invalid GUID({:?})", err),
        }
    }
}

impl std::fmt::Display for Guid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_str() {
            Ok(s) => write!(f, "{:?}", s),
            Err(err) => write!(f, "Invalid GUID({:?})", err),
        }
    }
}

impl TryFrom<Vec<u8>> for Guid {
    type Error = anyhow::Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        ensure!(&bytes[..3] == b"OK ");
        ensure!(&bytes[Self::LENGTH - 2..] == b"\r\n");
        Ok(Self(bytes))
    }
}
