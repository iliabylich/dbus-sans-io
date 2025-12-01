use std::collections::HashMap;

use crate::{
    messages::helpers::{body_is, interface_is, message_is, type_is, value_is},
    types::{CompleteType, Message, Value},
};
use anyhow::Result;

#[derive(Debug)]
pub(crate) struct PropertiesChanged {
    path: String,
    interface: String,
    changes: HashMap<String, Value>,
}

impl PropertiesChanged {
    pub(crate) fn try_parse(message: &Message) -> Result<Self> {
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
            changes.insert(key.to_string(), *value.clone());
        }

        Ok(Self {
            path: path.to_string(),
            interface: interface.to_string(),
            changes,
        })
    }
}
