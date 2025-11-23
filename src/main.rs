use anyhow::{Context as _, Result, bail};
use std::{
    io::{ErrorKind, Read as _, Write},
    os::unix::net::UnixStream,
};

mod io_operation;
pub use io_operation::{IoOperation, IoReader, IoRoundtrip, IoWriter};

mod readers;
pub(crate) use readers::{DynamicSizeReader, FixedSizeReader, MessageReader};

mod fixed_size_writer;
pub(crate) use fixed_size_writer::FixedSizeWriter;

mod serial;
pub(crate) use serial::Serial;

mod message_type;
pub use message_type::MessageType;

mod header_field;
pub use header_field::HeaderField;

mod flags;
pub use flags::Flags;

mod message;
pub use message::Message;

mod parsers;
pub use parsers::MessageParser;

use crate::fsm::{AuthFSM, AuthNextAction};

mod fsm;
mod guid;

struct MessageBuilder {
    data: Vec<u8>,
}

impl MessageBuilder {
    const LITTLE_ENDIAN: u8 = b'l';
    const PROTOCOL_VERSION: u8 = 1;

    fn new(message_type: MessageType, flags: u8, body_len: u32) -> Self {
        let mut header = Vec::new();
        header.push(Self::LITTLE_ENDIAN);
        header.push(message_type as u8);
        header.push(flags);
        header.push(Self::PROTOCOL_VERSION);
        header.extend_from_slice(&body_len.to_le_bytes());
        header.extend_from_slice(&0u32.to_le_bytes()); // serial placeholder
        header.extend_from_slice(&0u32.to_le_bytes()); // length placeholder

        Self { data: header }
    }

    fn push_u32(&mut self, n: u32) {
        self.data.extend_from_slice(&n.to_le_bytes());
    }

    fn push_binary_string(&mut self, s: &[u8]) {
        self.push_u32(s.len() as u32);
        self.data.extend_from_slice(s);
        self.data.push(0); // NULL EOS
    }

    fn push_signature(&mut self, sig: &[u8]) {
        self.data.push(sig.len() as u8);
        self.data.extend_from_slice(sig);
        self.data.push(0); // NULL EOS
    }

    fn align(&mut self) {
        while self.data.len() % 8 != 0 {
            self.data.push(0);
        }
    }

    fn add_string_field(&mut self, field: HeaderField, value: &[u8]) {
        self.align();
        self.data.push(field as u8);
        self.push_signature(b"s");
        self.push_binary_string(value);
    }

    fn add_object_path_field(&mut self, field: HeaderField, value: &[u8]) {
        self.align();
        self.data.push(field as u8);
        self.push_signature(b"o");
        self.push_binary_string(value);
    }

    fn finalize(mut self, serial: u32) -> Vec<u8> {
        self.data[8..12].copy_from_slice(&serial.to_le_bytes());
        let len = (self.data.len() - 16) as u32;
        self.data[12..16].copy_from_slice(&len.to_le_bytes());
        self.align();

        self.data
    }
}

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

    fn write_all(&mut self, data: &[u8]) {
        if let Err(err) = self.stream.write_all(data) {
            panic!("failed to write {data:?}: {err:?}");
        }
    }

    fn auth(&mut self) -> Result<guid::GUID> {
        let mut fsm = AuthFSM::new();
        let mut buf = [0_u8; 50];

        loop {
            match fsm.next_action() {
                AuthNextAction::Read(bytes_needed) => {
                    match self.stream.read(&mut buf[..bytes_needed]) {
                        Ok(len) => {
                            fsm.done_reading(&buf[..len])?;
                        }
                        Err(err) if err.kind() == ErrorKind::WouldBlock => {
                            continue;
                        }
                        Err(err) => return Err(err.into()),
                    }
                }

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

    fn send_message(&mut self, builder: MessageBuilder) -> u32 {
        let serial = self.serial.increment_and_get();
        let message = builder.finalize(serial);
        self.write_all(&message);
        serial
    }

    fn send_hello(&mut self) -> u32 {
        let mut msg = MessageBuilder::new(MessageType::MethodCall, 0, 0);
        msg.add_object_path_field(HeaderField::Path, b"/org/freedesktop/DBus");
        msg.add_string_field(HeaderField::Destination, b"org.freedesktop.DBus");
        msg.add_string_field(HeaderField::Interface, b"org.freedesktop.DBus");
        msg.add_string_field(HeaderField::Member, b"Hello");

        self.send_message(msg)
    }

    fn read_message(&mut self) -> Result<Message> {
        let mut reader = MessageReader::new();

        loop {
            match reader.continue_reading(&mut self.stream)? {
                IoOperation::Finished(message) => return Ok(message),
                IoOperation::WouldBlock => {}
            }
        }
    }
}

fn main() {
    let mut dbus = Connection::new_session();
    dbg!(dbus.auth().unwrap());

    let hello_serial = dbus.send_hello();
    println!("Sent Hello with serial {}", hello_serial);

    let mut msg = dbus.read_message().unwrap();
    println!("{msg:?}");

    let unique_name = msg.body.read_str().unwrap();
    println!("Our unique bus name: {}", unique_name);

    println!("\nWaiting for more messages...");
    loop {
        let mut msg = dbus.read_message().unwrap();

        print!("Received {:?}", msg.message_type);
        if let Some(ref member) = msg.member {
            print!(" {}", member);
        }
        if let Some(ref interface) = msg.interface {
            print!(" (interface={})", interface);
        }
        println!(" serial={}, body_len={}", msg.serial, msg.body.data.len());

        if msg.message_type == MessageType::Signal && msg.body.data.len() > 0 {
            let signal_arg = msg.body.read_str().unwrap();
            println!("  Signal argument: {}", signal_arg);
        }

        return;
    }
}
