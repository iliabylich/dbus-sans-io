use crate::types::{Header, ObjectPath, Signature, Value};

#[derive(Debug, Default)]
pub(crate) struct Message {
    pub(crate) header: Header,

    pub(crate) member: Option<String>,
    pub(crate) interface: Option<String>,
    pub(crate) path: Option<ObjectPath>,
    pub(crate) error_name: Option<String>,
    pub(crate) reply_serial: Option<u32>,
    pub(crate) destination: Option<String>,
    pub(crate) sender: Option<String>,
    pub(crate) signature: Option<Signature>,
    pub(crate) unix_fds: Option<u32>,

    pub(crate) body: Vec<Value>,
}
