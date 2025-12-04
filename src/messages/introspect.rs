use crate::{
    messages::helpers::{body_is, interface_is, member_is, message_is, path_is},
    types::{Message, Value},
};
use anyhow::Result;

#[derive(Debug)]
pub(crate) struct IntrospectRequest {
    pub(crate) serial: u32,
    pub(crate) destination: String,
    pub(crate) path: String,
    pub(crate) sender: String,
}

impl IntrospectRequest {
    pub(crate) fn try_parse(message: &Message) -> Result<Self> {
        message_is!(
            message,
            Message::MethodCall {
                serial,
                path,
                member,
                interface: Some(interface),
                destination: Some(destination),
                sender: Some(sender),
                body,
                ..
            }
        );

        path_is!(path, "/");
        member_is!(member, "Introspect");
        interface_is!(interface, "org.freedesktop.DBus.Introspectable");
        body_is!(body, []);

        Ok(Self {
            serial: *serial,
            destination: destination.clone(),
            path: path.clone(),
            sender: sender.clone(),
        })
    }
}

pub(crate) struct IntrospectResponse {
    req: IntrospectRequest,
    xml: &'static str,
}

impl IntrospectResponse {
    pub(crate) fn new(req: IntrospectRequest, xml: &'static str) -> Self {
        Self { req, xml }
    }

    pub(crate) fn into_message(self) -> Message {
        Message::MethodReturn {
            serial: 15,
            reply_serial: self.req.serial,
            destination: Some(self.req.sender),
            sender: None,
            unix_fds: None,
            body: vec![Value::String(self.xml.to_string())],
        }
    }
}
