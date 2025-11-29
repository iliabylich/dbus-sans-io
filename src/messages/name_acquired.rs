use crate::types::{Message, MessageType, Value};

#[derive(Debug)]
pub(crate) struct NameAcquired {
    pub(crate) name: String,
    pub(crate) message: Message,
}

impl TryFrom<Message> for NameAcquired {
    type Error = Message;

    fn try_from(message: Message) -> Result<Self, Self::Error> {
        macro_rules! message_type_is {
            ($message:expr, $expected:expr) => {
                if $message.header.message_type != $expected {
                    return Err($message);
                }
            };
        }
        macro_rules! interface_is {
            ($message:expr, $interface:expr) => {
                if $message.interface.is_none()
                    || $message.interface.as_ref().is_some_and(|v| v != $interface)
                {
                    return Err($message);
                }
            };
        }
        macro_rules! path_is {
            ($message:expr, $path:expr) => {
                if $message.path.is_none() || $message.path.as_ref().is_some_and(|v| v != $path) {
                    return Err($message);
                }
            };
        }

        message_type_is!(message, MessageType::Signal);
        interface_is!(message, "org.freedesktop.DBus");
        path_is!(message, b"/org/freedesktop/DBus");

        let mut body = message.body.iter();

        let Some(name) = body.next() else {
            return Err(message);
        };
        let Value::String(name) = name else {
            return Err(message);
        };

        Ok(Self {
            name: name.clone(),
            message,
        })
    }
}
