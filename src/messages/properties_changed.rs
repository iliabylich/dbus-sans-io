use crate::{
    body_is, interface_is, message_is, type_is,
    types::{CompleteType, Message, Value},
    value_is,
};
use anyhow::Result;
use std::{borrow::Cow, collections::HashMap};

#[derive(Debug)]
pub struct PropertiesChanged<'a> {
    pub path: Cow<'a, str>,
    pub interface: Cow<'a, str>,
    pub changes: HashMap<Cow<'a, str>, Value>,
}

impl<'a> TryFrom<&'a Message> for PropertiesChanged<'a> {
    type Error = anyhow::Error;

    fn try_from(message: &'a Message) -> Result<Self> {
        message_is!(
            message,
            Message::Signal {
                path,
                interface,
                body,
                ..
            }
        );

        interface_is!(interface, "org.freedesktop.DBus.Properties");
        body_is!(
            body,
            [Value::String(interface), Value::Array(item_t, items), _]
        );
        type_is!(item_t, CompleteType::DictEntry(key_t, value_t));
        type_is!(&**key_t, CompleteType::String);
        type_is!(&**value_t, CompleteType::Variant);

        let mut changes = HashMap::new();
        for item in items {
            value_is!(item, Value::DictEntry(key, value));
            value_is!(&**key, Value::String(key));
            value_is!(&**value, Value::Variant(value));
            changes.insert(Cow::Borrowed(key.as_str()), *value.clone());
        }

        Ok(Self {
            path: path.clone(),
            interface: Cow::Borrowed(interface),
            changes,
        })
    }
}
