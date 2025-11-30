use crate::types::{Message, Value};

#[derive(Debug)]
pub(crate) struct NameAcquired {
    pub(crate) name: String,
}

impl TryFrom<Message> for NameAcquired {
    type Error = Message;

    fn try_from(message: Message) -> Result<Self, Self::Error> {
        macro_rules! message_type_is {
            ($message:expr, $expected:ident) => {
                if !matches!($message, Message::$expected { .. }) {
                    return Err($message);
                }
            };
        }
        macro_rules! interface_is {
            ($message:expr, $interface:expr) => {
                if $message.interface() != Some($interface) {
                    return Err($message);
                }
            };
        }
        macro_rules! path_is {
            ($message:expr, $path:expr) => {
                if $message.path() != Some($path) {
                    return Err($message);
                }
            };
        }

        message_type_is!(message, Signal);
        interface_is!(message, "org.freedesktop.DBus");
        path_is!(message, b"/org/freedesktop/DBus");

        let mut body = message.body().iter();

        let Some(name) = body.next() else {
            return Err(message);
        };
        let Value::String(name) = name else {
            return Err(message);
        };

        Ok(Self { name: name.clone() })
    }
}
