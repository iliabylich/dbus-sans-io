use crate::types::{CompleteType, Message, Value};
use std::borrow::Cow;

pub struct ShowNotification {
    pub header: String,
    pub body: String,
}

impl ShowNotification {
    pub fn new(header: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            header: header.into(),
            body: body.into(),
        }
    }
}

impl From<ShowNotification> for Message {
    fn from(value: ShowNotification) -> Message {
        Message::MethodCall {
            serial: 0,
            path: Cow::Borrowed("/org/freedesktop/Notifications"),
            member: Cow::Borrowed("Notify"),
            interface: Some(Cow::Borrowed("org.freedesktop.Notifications")),
            destination: Some(Cow::Borrowed("org.freedesktop.Notifications")),
            sender: None,
            unix_fds: None,
            body: vec![
                Value::String(String::from("")),
                Value::UInt32(1),
                Value::String(String::from("")),
                Value::String(value.header),
                Value::String(value.body),
                Value::Array(CompleteType::String, vec![]),
                Value::Array(
                    CompleteType::DictEntry(
                        Box::new(CompleteType::String),
                        Box::new(CompleteType::Variant),
                    ),
                    vec![],
                ),
                Value::Int32(1_000),
            ],
        }
    }
}
