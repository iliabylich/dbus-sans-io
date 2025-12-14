use crate::{
    body_is, interface_is, member_is, message_is, path_is,
    types::{Message, Value},
};
use anyhow::Result;
use std::borrow::Cow;

#[derive(Debug)]
pub struct IntrospectRequest {
    pub serial: u32,
    pub destination: Cow<'static, str>,
    pub path: Cow<'static, str>,
    pub sender: Cow<'static, str>,
}

impl TryFrom<&Message> for IntrospectRequest {
    type Error = anyhow::Error;

    fn try_from(message: &Message) -> Result<Self> {
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

pub struct IntrospectResponse {
    req: IntrospectRequest,
    xml: &'static str,
}

impl IntrospectResponse {
    pub fn new(req: IntrospectRequest, xml: &'static str) -> Self {
        Self { req, xml }
    }
}

impl From<IntrospectResponse> for Message {
    fn from(value: IntrospectResponse) -> Message {
        Message::MethodReturn {
            serial: 0,
            reply_serial: value.req.serial,
            destination: Some(value.req.sender),
            sender: None,
            unix_fds: None,
            body: vec![Value::String(value.xml.to_string())],
        }
    }
}
