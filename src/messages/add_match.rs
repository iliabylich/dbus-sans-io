use crate::types::{Message, Value};

pub struct AddMatch {
    path: String,
}

impl AddMatch {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }
}

impl From<AddMatch> for Message {
    fn from(value: AddMatch) -> Message {
        Message::MethodCall {
            serial: 0,
            path: String::from("/org/freedesktop/DBus"),
            member: "AddMatch".to_string(),
            interface: Some(String::from("org.freedesktop.DBus")),
            destination: Some(String::from("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![Value::String(format!(
                "type='signal',interface='org.freedesktop.DBus.Properties',member='PropertiesChanged',path='{}'",
                value.path
            ))],
        }
    }
}
