use crate::{
    body_is, interface_is, message_is, path_is,
    types::{Message, Value},
};
use anyhow::Result;

#[derive(Debug)]
pub struct NameAcquired {
    pub name: String,
}

impl TryFrom<&Message> for NameAcquired {
    type Error = anyhow::Error;

    fn try_from(message: &Message) -> Result<Self> {
        message_is!(
            message,
            Message::Signal {
                path,
                interface,
                body,
                ..
            }
        );

        interface_is!(interface, "org.freedesktop.DBus");
        path_is!(path, "/org/freedesktop/DBus");
        body_is!(body, [Value::String(name)]);

        Ok(Self { name: name.clone() })
    }
}
