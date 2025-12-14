use crate::types::{Message, Value};
use std::borrow::Cow;

pub struct AddMatch {
    path: Cow<'static, str>,
}

impl AddMatch {
    pub fn new(path: Cow<'static, str>) -> Self {
        Self { path }
    }
}

impl From<AddMatch> for Message {
    fn from(value: AddMatch) -> Message {
        Message::MethodCall {
            serial: 0,
            path: Cow::Borrowed("/org/freedesktop/DBus"),
            member: Cow::Borrowed("AddMatch"),
            interface: Some(Cow::Borrowed("org.freedesktop.DBus")),
            destination: Some(Cow::Borrowed("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![Value::String(format!(
                "type='signal',interface='org.freedesktop.DBus.Properties',member='PropertiesChanged',path='{}'",
                value.path
            ))],
        }
    }
}
