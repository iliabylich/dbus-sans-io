use anyhow::{Result, bail, ensure};
use libc::{POLLERR, POLLIN, POLLOUT, poll, pollfd};
use std::os::{fd::AsRawFd, unix::net::UnixStream};

mod decoders;
mod encoders;
mod non_blocking_unix_stream;
mod serial;
mod types;

use crate::{
    encoders::MessageEncoder,
    fsm::{AuthFSM, FSMSatisfy, FSMWants, FullFSM, Output, ReaderFSM, WriterFSM},
    non_blocking_unix_stream::NonBlockingUnixStream,
    serial::Serial,
    types::{Flags, GUID, Header, Message, MessageType, ObjectPath, Value},
};

mod fsm;

struct Connection {
    stream: NonBlockingUnixStream,
    serial: Serial,
    fsm: FullFSM,
}

impl AsRawFd for Connection {
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        self.stream.as_raw_fd()
    }
}

impl Connection {
    fn new_session() -> Self {
        let address =
            std::env::var("DBUS_SESSION_BUS_ADDRESS").expect("no DBUS_SESSION_BUS_ADDRESS");
        let (_, path) = address.split_once("=").expect("no = separator");
        let stream = UnixStream::connect(path).expect("failed to create unix socket");
        stream.set_nonblocking(true).unwrap();

        Self {
            stream: NonBlockingUnixStream::new(stream),
            serial: Serial::zero(),
            fsm: FullFSM::new(),
        }
    }

    fn poll_enqueue(&mut self, mut message: Message) -> Result<Message> {
        let serial = self.serial.increment_and_get();
        message.header.serial = serial;
        let buf = MessageEncoder::encode(&message)?;

        self.fsm.enqueue(buf)?;
        Ok(message)
    }

    fn poll_auth_events(&mut self) -> i16 {
        match self.fsm.wants() {
            FSMWants::Read(_) => POLLIN,
            FSMWants::Write(_) => POLLOUT,
            FSMWants::Nothing => unreachable!(),
        }
    }

    fn poll_auth(&mut self, readable: bool, writable: bool) -> Result<Option<GUID>> {
        loop {
            let mut did = false;

            if writable && let FSMWants::Write(buf) = self.fsm.wants() {
                match self.stream.write(buf)? {
                    Some(len) => {
                        did = true;
                        match self.fsm.satisfy(FSMSatisfy::Write { len })? {
                            Output::GUID(guid) => return Ok(Some(guid)),
                            Output::Message(_) => unreachable!(),
                            Output::NothingYet => {}
                        }
                    }
                    None => {}
                }
            }

            if readable && let FSMWants::Read(buf) = self.fsm.wants() {
                match self.stream.read(buf)? {
                    Some(len) => {
                        did = true;
                        match self.fsm.satisfy(FSMSatisfy::Read { len })? {
                            Output::GUID(_) => unreachable!(),
                            Output::Message(_) => unreachable!(),
                            Output::NothingYet => {}
                        }
                    }
                    None => {}
                }
            }

            if !did {
                break;
            }
        }

        Ok(None)
    }

    fn poll_read_write_events(&mut self) -> i16 {
        match self.fsm.wants() {
            FSMWants::Read(_) => POLLIN,
            FSMWants::Write(_) => POLLOUT,
            FSMWants::Nothing => unreachable!(),
        }
    }

    fn poll_read_write(&mut self, readable: bool, writable: bool) -> Result<Option<Message>> {
        loop {
            let mut did = false;

            if writable && let FSMWants::Write(buf) = self.fsm.wants() {
                match self.stream.write(buf)? {
                    Some(len) => {
                        did = true;
                        match self.fsm.satisfy(FSMSatisfy::Write { len })? {
                            Output::GUID(_) => unreachable!(),
                            Output::Message(_) => unreachable!(),
                            Output::NothingYet => {}
                        }
                    }
                    None => {}
                }
            }

            if readable && let FSMWants::Read(buf) = self.fsm.wants() {
                match self.stream.read(buf)? {
                    Some(len) => {
                        did = true;
                        match self.fsm.satisfy(FSMSatisfy::Read { len })? {
                            Output::GUID(_) => unreachable!(),
                            Output::Message(message) => return Ok(Some(message)),
                            Output::NothingYet => {}
                        }
                    }
                    None => {}
                }
            }

            if !did {
                break;
            }
        }

        Ok(None)
    }

    fn blocking_auth(&mut self) -> Result<GUID> {
        let mut fsm = AuthFSM::new();

        loop {
            match fsm.wants() {
                FSMWants::Read(buf) => match self.stream.read(buf)? {
                    Some(len) => {
                        fsm.satisfy(FSMSatisfy::Read { len })?;
                    }
                    None => {}
                },

                FSMWants::Write(bytes) => match self.stream.write(bytes)? {
                    Some(len) => {
                        if let Some(guid) = fsm.satisfy(FSMSatisfy::Write { len })? {
                            return Ok(guid);
                        }
                    }
                    None => {}
                },

                FSMWants::Nothing => {}
            }
        }
    }

    fn blocking_send_message(&mut self, message: &mut Message) -> Result<u32> {
        let serial = self.serial.increment_and_get();
        message.header.serial = serial;
        let buf = MessageEncoder::encode(message)?;

        let mut fsm = WriterFSM::new();
        fsm.enqueue(buf);

        loop {
            match fsm.wants() {
                FSMWants::Nothing => break,
                FSMWants::Write(buf) => match self.stream.write(buf)? {
                    Some(len) => {
                        fsm.satisfy(FSMSatisfy::Write { len })?;
                    }
                    None => continue,
                },
                FSMWants::Read(_) => unreachable!(),
            }
        }

        Ok(serial)
    }

    fn hello() -> Message {
        Message {
            header: Header {
                message_type: MessageType::MethodCall,
                flags: Flags { byte: 0 },
                serial: 0,
                body_len: 0,
            },
            member: Some(String::from("Hello")),
            interface: Some(String::from("org.freedesktop.DBus")),
            path: Some(ObjectPath(b"/org/freedesktop/DBus".to_vec())),
            error_name: None,
            reply_serial: None,
            destination: Some(String::from("org.freedesktop.DBus")),
            sender: None,
            signature: None,
            unix_fds: None,
            body: vec![],
        }
    }

    fn send_hello(&mut self) -> Result<u32> {
        let mut message = Self::hello();

        self.blocking_send_message(&mut message)
    }

    fn read_message(&mut self) -> Result<Message> {
        let mut fsm = ReaderFSM::new();

        loop {
            match fsm.wants() {
                FSMWants::Read(buf) => match self.stream.read(buf)? {
                    Some(len) => {
                        if let Some(message) = fsm.satisfy(FSMSatisfy::Read { len })? {
                            return Ok(message);
                        }
                    }

                    None => {
                        continue;
                    }
                },

                FSMWants::Write(_) => unreachable!(),
                FSMWants::Nothing => unreachable!(),
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

#[allow(dead_code)]
fn main_blocking(mut dbus: Connection) {
    let _serial = dbus.blocking_auth().unwrap();
    dbg!(dbus.send_hello().unwrap());
    loop {
        let msg = dbus.read_message().unwrap();

        println!("Received {:?}", msg);
    }
}

#[allow(dead_code)]
fn main_poll(mut dbus: Connection) {
    let mut fds = [pollfd {
        fd: dbus.as_raw_fd(),
        events: POLLIN | POLLOUT,
        revents: 0,
    }];

    fn do_poll(fds: &mut [pollfd; 1]) -> (bool, bool) {
        println!("Polling...");
        let res = unsafe { poll(fds.as_mut_ptr(), 1, -1) };
        assert!(res == 1);
        let readable = fds[0].revents & POLLIN != 0;
        let writable = fds[0].revents & POLLOUT != 0;
        assert_eq!(fds[0].revents & POLLERR, 0);
        println!("{readable} {writable}");
        (readable, writable)
    }

    loop {
        fds[0].events = dbus.poll_auth_events();
        let (readable, writable) = do_poll(&mut fds);

        if let Some(guid) = dbus.poll_auth(readable, writable).unwrap() {
            println!("GUID: {}", guid.as_str().unwrap());
            break;
        }
    }
    let _sent = dbus.poll_enqueue(Connection::hello()).unwrap();
    loop {
        fds[0].events = dbus.poll_read_write_events();
        let (readable, writable) = do_poll(&mut fds);

        if let Some(message) = dbus.poll_read_write(readable, writable).unwrap() {
            println!("Received: {:?}", message);
        }
    }
}

fn main() {
    let dbus = Connection::new_session();
    // main_blocking(dbus);
    main_poll(dbus);
}
