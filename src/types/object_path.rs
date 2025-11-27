#[derive(Clone)]
pub(crate) struct ObjectPath(Vec<u8>);

impl ObjectPath {
    pub(crate) fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub(crate) fn as_str_lossy(&self) -> String {
        String::from_utf8_lossy(&self.0).into_owned()
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl std::fmt::Debug for ObjectPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_str_lossy())
    }
}
