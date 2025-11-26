#[derive(Clone)]
pub struct ObjectPath(pub Vec<u8>);

impl ObjectPath {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn as_str_lossy(&self) -> String {
        String::from_utf8_lossy(&self.0).into_owned()
    }
}

impl std::fmt::Debug for ObjectPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_str_lossy())
    }
}
