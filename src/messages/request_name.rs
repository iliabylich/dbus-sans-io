use crate::types::{Message, Value};

pub struct RequestName {
    name: String,
}

impl RequestName {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl From<RequestName> for Message {
    fn from(value: RequestName) -> Message {
        Message::MethodCall {
            serial: 0,
            path: String::from("/org/freedesktop/DBus"),
            member: String::from("RequestName"),
            interface: Some(String::from("org.freedesktop.DBus")),
            destination: Some(String::from("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![Value::String(value.name), Value::UInt32(7)],
        }
    }
}
