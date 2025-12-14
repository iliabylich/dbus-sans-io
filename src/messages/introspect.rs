use crate::{
    body_is, interface_is, member_is, message_is, path_is,
    types::{Message, Value},
};
use anyhow::Result;
use std::borrow::Cow;

#[derive(Debug)]
pub struct IntrospectRequest<'a> {
    pub serial: u32,
    pub destination: Cow<'a, str>,
    pub path: Cow<'a, str>,
    pub sender: Cow<'a, str>,
}

impl<'a> TryFrom<&'a Message> for IntrospectRequest<'a> {
    type Error = anyhow::Error;

    fn try_from(message: &'a Message) -> Result<Self> {
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

pub struct IntrospectResponse<'a> {
    req: IntrospectRequest<'a>,
    xml: &'static str,
}

impl<'a> IntrospectResponse<'a> {
    pub fn new(req: IntrospectRequest<'a>, xml: &'static str) -> Self {
        Self { req, xml }
    }
}

impl<'a> From<IntrospectResponse<'a>> for Message {
    fn from(value: IntrospectResponse<'a>) -> Message {
        Message::MethodReturn {
            serial: 0,
            reply_serial: value.req.serial,
            destination: Some(Cow::Owned(value.req.sender.to_string())),
            sender: None,
            unix_fds: None,
            body: vec![Value::String(value.xml.to_string())],
        }
    }
}
