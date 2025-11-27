use anyhow::Result;
use libc::{POLLERR, POLLIN, POLLOUT, poll, pollfd};
use std::{
    io::{Read as _, Write as _},
    os::{fd::AsRawFd, unix::net::UnixStream},
};

mod decoders;
mod encoders;
mod fsm;
mod non_blocking_unix_stream;
mod serial;
mod types;

use crate::{
    fsm::{AuthFSM, AuthWants, ReaderFSM, WriterFSM},
    non_blocking_unix_stream::NonBlockingUnixStream,
    serial::Serial,
    types::{Flags, GUID, Header, Message, MessageType, ObjectPath},
};

fn conn() -> UnixStream {
    let address = std::env::var("DBUS_SESSION_BUS_ADDRESS").expect("no DBUS_SESSION_BUS_ADDRESS");
    let (_, path) = address.split_once("=").expect("no = separator");
    UnixStream::connect(path).expect("failed to create unix socket")
}

struct NonBlockingConnection {
    stream: NonBlockingUnixStream,
    serial: Serial,

    auth: AuthFSM,
    reader: ReaderFSM,
    writer: WriterFSM,
}

impl AsRawFd for NonBlockingConnection {
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        self.stream.as_raw_fd()
    }
}

impl NonBlockingConnection {
    fn new_session() -> Self {
        let stream = conn();
        stream.set_nonblocking(true).unwrap();

        Self {
            stream: NonBlockingUnixStream::new(stream),
            serial: Serial::zero(),

            auth: AuthFSM::new(),
            reader: ReaderFSM::new(),
            writer: WriterFSM::new(),
        }
    }

    fn poll_enqueue(&mut self, mut message: Message) -> Result<Message> {
        let serial = self.serial.increment_and_get();
        message.header.serial = serial;

        self.writer.enqueue(&message)?;
        Ok(message)
    }

    fn poll_auth_events(&mut self) -> i16 {
        match self.auth.wants() {
            AuthWants::Read(_) => POLLIN,
            AuthWants::Write(_) => POLLOUT,
        }
    }

    fn poll_auth(&mut self, readable: bool, writable: bool) -> Result<Option<GUID>> {
        loop {
            let mut did = false;

            if writable
                && let AuthWants::Write(buf) = self.auth.wants()
                && let Some(len) = self.stream.write(buf)?
            {
                did = true;
                if let Some(guid) = self.auth.satisfy_write(len)? {
                    return Ok(Some(guid));
                }
            }

            if readable
                && let AuthWants::Read(buf) = self.auth.wants()
                && let Some(len) = self.stream.read(buf)?
            {
                did = true;
                self.auth.satisfy_read(len)?;
            }

            if !did {
                break;
            }
        }

        Ok(None)
    }

    fn poll_read_write_events(&mut self) -> i16 {
        let mut out = POLLIN;
        if self.writer.wants_write().is_some() {
            out |= POLLOUT;
        }
        out
    }

    fn poll_read_one_message(&mut self) -> Result<Option<Message>> {
        loop {
            let buf = self.reader.wants_read();
            let Some(len) = self.stream.read(buf)? else {
                return Ok(None);
            };

            if let Some(message) = self.reader.satisfy(len)? {
                return Ok(Some(message));
            }
        }
    }

    fn poll_write_to_end(&mut self) -> Result<()> {
        loop {
            let Some(buf) = self.writer.wants_write() else {
                break;
            };
            let Some(len) = self.stream.write(buf)? else {
                break;
            };
            self.writer.satisfy(len)?;
        }
        Ok(())
    }
}

struct BlockingConnection {
    stream: UnixStream,
    serial: Serial,

    auth: AuthFSM,
    reader: ReaderFSM,
    writer: WriterFSM,
}

impl BlockingConnection {
    fn new_session() -> Self {
        let stream = conn();

        Self {
            stream,
            serial: Serial::zero(),

            auth: AuthFSM::new(),
            reader: ReaderFSM::new(),
            writer: WriterFSM::new(),
        }
    }

    fn blocking_auth(&mut self) -> Result<GUID> {
        loop {
            match self.auth.wants() {
                AuthWants::Read(buf) => {
                    let len = self.stream.read(buf)?;
                    self.auth.satisfy_read(len)?;
                }

                AuthWants::Write(bytes) => {
                    let len = self.stream.write(bytes)?;
                    if let Some(guid) = self.auth.satisfy_write(len)? {
                        return Ok(guid);
                    }
                }
            }
        }
    }

    fn blocking_send_message(&mut self, mut message: Message) -> Result<Message> {
        message.header.serial = self.serial.increment_and_get();
        self.writer.enqueue(&message)?;

        loop {
            let Some(buf) = self.writer.wants_write() else {
                break;
            };
            let len = self.stream.write(buf)?;
            self.writer.satisfy(len)?;
        }

        Ok(message)
    }

    fn blocking_read_message(&mut self) -> Result<Message> {
        loop {
            let buf = self.reader.wants_read();
            let len = self.stream.read(buf)?;
            if let Some(message) = self.reader.satisfy(len)? {
                return Ok(message);
            }
        }
    }
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
        path: Some(ObjectPath::new(b"/org/freedesktop/DBus".to_vec())),
        error_name: None,
        reply_serial: None,
        destination: Some(String::from("org.freedesktop.DBus")),
        sender: None,
        signature: None,
        unix_fds: None,
        body: vec![],
    }
}

#[allow(dead_code)]
fn main_blocking() {
    let mut dbus = BlockingConnection::new_session();
    let _guid = dbus.blocking_auth().unwrap();
    let _serial = dbus.blocking_send_message(hello()).unwrap();
    loop {
        let msg = dbus.blocking_read_message().unwrap();

        println!("Received {:?}", msg);
    }
}

#[allow(dead_code)]
fn main_poll() {
    let mut dbus = NonBlockingConnection::new_session();

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
    let _sent = dbus.poll_enqueue(hello()).unwrap();
    loop {
        fds[0].events = dbus.poll_read_write_events();
        let (readable, writable) = do_poll(&mut fds);

        if writable {
            dbus.poll_write_to_end().unwrap();
        }

        if readable {
            while let Some(message) = dbus.poll_read_one_message().unwrap() {
                println!("Received: {:?}", message);
            }
        }
    }
}

fn main() {
    // main_blocking();
    main_poll();
}
