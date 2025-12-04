use crate::types::{Message, Value};

pub(crate) struct RequestName {
    name: String,
}

impl RequestName {
    pub(crate) fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub(crate) fn into_message(self) -> Message {
        Message::MethodCall {
            serial: 0,
            path: String::from("/org/freedesktop/DBus"),
            member: String::from("RequestName"),
            interface: Some(String::from("org.freedesktop.DBus")),
            destination: Some(String::from("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![Value::String(self.name), Value::UInt32(7)],
        }
    }
}
