use anyhow::{Result, bail, ensure};
use std::{
    io::{ErrorKind, Read as _, Write},
    os::unix::net::UnixStream,
};

mod io_operation;
pub use io_operation::{IoOperation, IoReader, IoRoundtrip, IoWriter};

mod serial;
pub(crate) use serial::Serial;

mod decoders;
mod encoders;
mod types;

use crate::{
    encoders::MessageEncoder,
    fsm::{AuthFSM, AuthNextAction, ReaderFSM, ReaderNextAction},
    types::{Flags, Message, MessageSignature, MessageType, ObjectPath, Value},
};

mod fsm;
mod guid;

struct Connection {
    stream: UnixStream,
    serial: Serial,
}

impl Connection {
    fn new_session() -> Self {
        let address =
            std::env::var("DBUS_SESSION_BUS_ADDRESS").expect("no DBUS_SESSION_BUS_ADDRESS");
        let (_, path) = address.split_once("=").expect("no = separator");
        println!("{path:?}");
        let stream = UnixStream::connect(path).expect("failed to create unix socket");
        stream.set_nonblocking(true).unwrap();

        Self {
            stream,
            serial: Serial::zero(),
        }
    }

    fn write_all(&mut self, buf: &[u8]) {
        if let Err(err) = self.stream.write_all(buf) {
            panic!("failed to write {buf:?}: {err:?}");
        }
    }

    fn auth(&mut self) -> Result<guid::GUID> {
        let mut fsm = AuthFSM::new();

        loop {
            match fsm.next_action() {
                AuthNextAction::Read(buf) => match self.stream.read(buf) {
                    Ok(len) => {
                        fsm.done_reading(len)?;
                    }
                    Err(err) if err.kind() == ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(err) => return Err(err.into()),
                },

                AuthNextAction::Write(bytes) => match self.stream.write(bytes) {
                    Ok(len) => {
                        fsm.done_writing(len)?;
                    }
                    Err(err) if err.kind() == ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(err) => return Err(err.into()),
                },

                AuthNextAction::Done(guid) => return Ok(guid),
            }
        }
    }

    fn send_message(&mut self, message: &mut Message) -> Result<u32> {
        let serial = self.serial.increment_and_get();
        message.serial = serial;
        let message = MessageEncoder::encode(message)?;
        self.write_all(&message);
        Ok(serial)
    }

    fn send_hello(&mut self) -> Result<u32> {
        let mut message = Message {
            message_type: MessageType::MethodCall,
            flags: Flags { byte: 0 },
            serial: 0,
            member: Some(String::from("Hello")),
            interface: Some(String::from("org.freedesktop.DBus")),
            path: Some(ObjectPath(b"/org/freedesktop/DBus".to_vec())),
            error_name: None,
            reply_serial: None,
            destination: Some(String::from("org.freedesktop.DBus")),
            sender: None,
            body_signature: MessageSignature(vec![]),
            unix_fds: None,
            body: vec![],
        };

        self.send_message(&mut message)
    }

    fn read_message(&mut self) -> Result<Message> {
        let mut fsm = ReaderFSM::new();

        loop {
            match fsm.next_action() {
                ReaderNextAction::Read(buf) => match self.stream.read(buf) {
                    Ok(len) => {
                        fsm.done_reading(len)?;
                    }

                    Err(err) if err.kind() == ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(err) => return Err(err.into()),
                },

                ReaderNextAction::Message(message) => {
                    return Ok(message);
                }
            }
        }
    }
}

pub trait FromMessage: Sized {
    fn from_message(message: Message) -> Result<Self>;
}

pub struct NameAcquired {
    pub name: String,
}
impl FromMessage for NameAcquired {
    fn from_message(message: Message) -> Result<Self> {
        ensure!(message.body.len() == 1);
        let name = message.body.into_iter().next().unwrap();
        let Value::String(name) = name else {
            bail!("NameAcquired: expected String, got {name:?}");
        };
        Ok(Self { name })
    }
}

fn main() {
    let mut dbus = Connection::new_session();
    dbg!(dbus.auth().unwrap());

    dbus.send_hello();

    loop {
        let msg = dbus.read_message().unwrap();

        println!("Received {:?}", msg);
    }
}
