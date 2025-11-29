use crate::types::{Header, Signature, Value};

#[derive(Default)]
pub(crate) struct Message {
    pub(crate) header: Header,

    pub(crate) member: Option<String>,
    pub(crate) interface: Option<String>,
    pub(crate) path: Option<Vec<u8>>,
    pub(crate) error_name: Option<String>,
    pub(crate) reply_serial: Option<u32>,
    pub(crate) destination: Option<String>,
    pub(crate) sender: Option<String>,
    pub(crate) signature: Option<Signature>,
    pub(crate) unix_fds: Option<u32>,

    pub(crate) body: Vec<Value>,
}

impl Message {
    pub(crate) fn compute_body_signature(&self) -> Signature {
        Signature {
            items: self.body.iter().map(|item| item.complete_type()).collect(),
        }
    }
}

impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let path = self.path.as_ref().map(|path| String::from_utf8_lossy(path));

        f.debug_struct("Message")
            .field("header", &self.header)
            .field("member", &self.member)
            .field("interface", &self.interface)
            .field("path", &path)
            .field("error_name", &self.error_name)
            .field("reply_serial", &self.reply_serial)
            .field("destination", &self.destination)
            .field("sender", &self.sender)
            .field("signature", &self.signature)
            .field("unix_fds", &self.unix_fds)
            .field("body", &self.body)
            .finish()
    }
}
