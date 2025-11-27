use anyhow::ensure;

#[derive(Clone, PartialEq, Eq, Default)]
pub(crate) struct GUID(pub(crate) Vec<u8>);

impl GUID {
    pub(crate) const LENGTH: usize = 37;

    pub(crate) fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.0[3..Self::LENGTH - 2])
    }
}

impl std::fmt::Debug for GUID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_str() {
            Ok(s) => write!(f, "{:?}", s),
            Err(err) => write!(f, "Invalid GUID({:?})", err),
        }
    }
}

impl std::fmt::Display for GUID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_str() {
            Ok(s) => write!(f, "{:?}", s),
            Err(err) => write!(f, "Invalid GUID({:?})", err),
        }
    }
}

impl TryFrom<Vec<u8>> for GUID {
    type Error = anyhow::Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        ensure!(&bytes[..3] == b"OK ");
        ensure!(&bytes[Self::LENGTH - 2..] == b"\r\n");
        Ok(Self(bytes))
    }
}
