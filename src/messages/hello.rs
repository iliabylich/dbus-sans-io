use crate::types::Message;

pub(crate) struct Hello;

impl Hello {
    pub(crate) fn into_message(self) -> Message {
        Message::MethodCall {
            serial: 0,
            path: String::from("/org/freedesktop/DBus"),
            member: String::from("Hello"),
            interface: Some(String::from("org.freedesktop.DBus")),
            destination: Some(String::from("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![],
        }
    }
}
