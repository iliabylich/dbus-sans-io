use crate::types::{Header, ObjectPath, Signature, Value};

#[derive(Debug, Default)]
pub struct Message {
    pub header: Header,

    pub member: Option<String>,
    pub interface: Option<String>,
    pub path: Option<ObjectPath>,
    pub error_name: Option<String>,
    pub reply_serial: Option<u32>,
    pub destination: Option<String>,
    pub sender: Option<String>,
    pub signature: Option<Signature>,
    pub unix_fds: Option<u32>,

    pub body: Vec<Value>,
}
