use anyhow::{Result, bail, ensure};
use libc::{POLLERR, POLLIN, POLLOUT, poll, pollfd};
use std::os::{fd::AsRawFd, unix::net::UnixStream};

mod decoders;
mod encoders;
mod non_blocking_unix_stream;
mod serial;
mod types;

use crate::{
    fsm::{AuthFSM, FSMSatisfy, FSMWants, ReaderFSM, ReaderWriterFSM, WriterFSM},
    non_blocking_unix_stream::NonBlockingUnixStream,
    serial::Serial,
    types::{Flags, GUID, Header, Message, MessageType, ObjectPath, Value},
};

mod fsm;

struct Connection {
    stream: NonBlockingUnixStream,
    serial: Serial,

    auth: AuthFSM,
    reader_writer: ReaderWriterFSM,
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

            auth: AuthFSM::new(),
            reader_writer: ReaderWriterFSM::new(),
        }
    }

    fn poll_enqueue(&mut self, mut message: Message) -> Result<Message> {
        let serial = self.serial.increment_and_get();
        message.header.serial = serial;

        self.reader_writer.enqueue(&message)?;
        Ok(message)
    }

    fn poll_auth_events(&mut self) -> i16 {
        match self.auth.wants() {
            FSMWants::Read(_) => POLLIN,
            FSMWants::Write(_) => POLLOUT,
            FSMWants::Nothing => unreachable!(),
        }
    }

    fn poll_auth(&mut self, readable: bool, writable: bool) -> Result<Option<GUID>> {
        loop {
            let mut did = false;

            if writable
                && let FSMWants::Write(buf) = self.auth.wants()
                && let Some(len) = self.stream.write(buf)?
            {
                did = true;
                if let Some(guid) = self.auth.satisfy(FSMSatisfy::Write { len })? {
                    return Ok(Some(guid));
                }
            }

            if readable
                && let FSMWants::Read(buf) = self.auth.wants()
                && let Some(len) = self.stream.read(buf)?
            {
                did = true;
                if self.auth.satisfy(FSMSatisfy::Read { len })?.is_some() {
                    unreachable!("auth.satisfy(read) never returns Some()");
                }
            }

            if !did {
                break;
            }
        }

        Ok(None)
    }

    fn poll_read_write_events(&mut self) -> i16 {
        match self.reader_writer.wants() {
            FSMWants::Read(_) => POLLIN,
            FSMWants::Write(_) => POLLOUT,
            FSMWants::Nothing => unreachable!(),
        }
    }

    fn poll_read_write(&mut self, readable: bool, writable: bool) -> Result<Option<Message>> {
        loop {
            let mut did = false;

            if writable
                && let FSMWants::Write(buf) = self.reader_writer.wants()
                && let Some(len) = self.stream.write(buf)?
            {
                did = true;
                if self
                    .reader_writer
                    .satisfy(FSMSatisfy::Write { len })?
                    .is_some()
                {
                    unreachable!();
                }
            }

            if readable
                && let FSMWants::Read(buf) = self.reader_writer.wants()
                && let Some(len) = self.stream.read(buf)?
            {
                did = true;
                if let Some(message) = self.reader_writer.satisfy(FSMSatisfy::Read { len })? {
                    return Ok(Some(message));
                }
            }

            if !did {
                break;
            }
        }

        Ok(None)
    }

    fn blocking_auth(&mut self) -> Result<GUID> {
        loop {
            match self.auth.wants() {
                FSMWants::Read(buf) => {
                    if let Some(len) = self.stream.read(buf)? {
                        self.auth.satisfy(FSMSatisfy::Read { len })?;
                    }
                }

                FSMWants::Write(bytes) => {
                    if let Some(len) = self.stream.write(bytes)?
                        && let Some(guid) = self.auth.satisfy(FSMSatisfy::Write { len })?
                    {
                        return Ok(guid);
                    }
                }

                FSMWants::Nothing => {}
            }
        }
    }

    fn blocking_send_message(&mut self, mut message: Message) -> Result<Message> {
        message.header.serial = self.serial.increment_and_get();

        let mut fsm = WriterFSM::new();
        fsm.enqueue(&message)?;

        loop {
            match fsm.wants() {
                FSMWants::Nothing => break,
                FSMWants::Write(buf) => {
                    if let Some(len) = self.stream.write(buf)? {
                        fsm.satisfy(FSMSatisfy::Write { len })?;
                    }
                }
                FSMWants::Read(_) => unreachable!(),
            }
        }

        Ok(message)
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

    fn blocking_read_message(&mut self) -> Result<Message> {
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
    let _guid = dbus.blocking_auth().unwrap();
    let _serial = dbus.blocking_send_message(Connection::hello()).unwrap();
    loop {
        let msg = dbus.blocking_read_message().unwrap();

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
