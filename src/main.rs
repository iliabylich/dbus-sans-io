use anyhow::{Context as _, Result, bail};
use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
};

mod io_operation;
pub use io_operation::{IoOperation, IoReader, IoWriter};

mod fixed_size_reader;
pub(crate) use fixed_size_reader::FixedSizeReader;

mod fixed_size_writer;
pub(crate) use fixed_size_writer::FixedSizeWriter;

mod header_reader;
pub(crate) use header_reader::{Header, HeaderReader};

mod message_type;
pub use message_type::MessageType;

mod header_fields;
pub use header_fields::HeaderField;

mod flags;
pub use flags::Flags;

struct Message {
    message_type: MessageType,
    flags: u8,
    serial: u32,
    member: Option<String>,
    interface: Option<String>,
    path: Option<String>,
    body: MessageParser,
}

struct MessageParser {
    data: Vec<u8>,
    pos: usize,
}

impl MessageParser {
    fn new(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }

    fn read_u8(&mut self) -> Result<u8> {
        let byte = self.data.get(self.pos).copied().context("EOF")?;
        self.pos += 1;
        Ok(byte)
    }

    fn read_u32(&mut self) -> Result<u32> {
        let value = u32::from_le_bytes([
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?,
        ]);
        Ok(value)
    }

    fn read_str(&mut self) -> Result<&str> {
        let len = self.read_u32()? as usize;
        let s = std::str::from_utf8(&self.data[self.pos..self.pos + len])
            .expect("invalid UTF-8 in string");
        self.pos += len + 1; // +1 for null terminator
        Ok(s)
    }
}

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
    serial: u32,
}

impl Connection {
    fn new_session() -> Self {
        let address =
            std::env::var("DBUS_SESSION_BUS_ADDRESS").expect("no DBUS_SESSION_BUS_ADDRESS");
        let (_, path) = address.split_once("=").expect("no = separator");
        println!("{path:?}");
        let mut stream = UnixStream::connect(path).expect("failed to create unix socket");
        // stream.set_nonblocking(true).unwrap();

        let written = stream.write(b"\0").expect("failed to write NULL");
        assert_eq!(written, 1);

        Self { stream, serial: 0 }
    }

    fn write_all(&mut self, data: &[u8]) {
        if let Err(err) = self.stream.write_all(data) {
            panic!("failed to write {data:?}: {err:?}");
        }
    }

    fn read_binary(&mut self, buf: &mut [u8]) -> usize {
        match self.stream.read(buf) {
            Ok(len) => len,
            Err(err) => {
                panic!("failed to read: {err:?}")
            }
        }
    }

    fn authenticate(&mut self) {
        self.write_all(b"AUTH EXTERNAL\r\n");

        let mut buf = [0u8; 256];
        let len = self.read_binary(&mut buf);
        assert_eq!(&buf[..len], b"DATA\r\n");

        self.write_all(b"DATA\r\n");

        let len = self.read_binary(&mut buf);
        let guid = buf[..len]
            .strip_prefix(b"OK ")
            .expect("no OK prefix in data response")
            .strip_suffix(b"\r\n")
            .expect("no \\r\\n suffix in data response");
        println!("GUID: {:?}", std::str::from_utf8(guid));

        self.write_all(b"BEGIN\r\n");
    }

    fn send_message(&mut self, builder: MessageBuilder) -> u32 {
        self.serial += 1;
        let message = builder.finalize(self.serial);
        self.write_all(&message);
        self.serial
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
        let mut header_reader = HeaderReader::new();
        let header = header_reader.continue_reading(&mut self.stream)?;

        let IoOperation::Finished(header) = header else {
            panic!("failed to read to end");
        };

        let message_type = header.message_type;
        let flags = header.flags;
        let body_len = header.body_len;
        let serial = header.serial;
        let headers_len = header.headers_len;

        let mut header_fields = vec![0u8; headers_len as usize];
        self.read_binary(&mut header_fields);

        // Parse header fields
        let mut member = None;
        let mut interface = None;
        let mut path = None;
        let mut pos = 0;
        while pos < header_fields.len() {
            // Align to 8-byte boundary from message start
            let absolute_pos = 16 + pos;
            let padding = (8 - (absolute_pos % 8)) % 8;
            pos += padding;

            if pos >= header_fields.len() {
                break;
            }

            let field_code = header_fields[pos];
            pos += 1;
            let sig_len = header_fields[pos];
            pos += 1;
            let signature = header_fields[pos];
            pos += 1;
            pos += 1; // skip signature null terminator

            match signature {
                b's' | b'o' => {
                    let str_len = u32::from_le_bytes([
                        header_fields[pos],
                        header_fields[pos + 1],
                        header_fields[pos + 2],
                        header_fields[pos + 3],
                    ]) as usize;
                    pos += 4;
                    let value =
                        String::from_utf8_lossy(&header_fields[pos..pos + str_len]).into_owned();
                    pos += str_len + 1; // +1 for null terminator

                    if let Ok(field) = HeaderField::try_from(field_code) {
                        match field {
                            HeaderField::Path => path = Some(value),
                            HeaderField::Interface => interface = Some(value),
                            HeaderField::Member => member = Some(value),
                            _ => {}
                        }
                    }
                }
                _ => break, // Skip unknown signatures for now
            }
        }

        // Skip padding to 8-byte boundary
        let padding = (8 - ((16 + headers_len as usize) % 8)) % 8;
        if padding > 0 {
            let mut pad = [0u8; 7];
            self.read_binary(&mut pad[..padding]);
        }

        let mut body = vec![0u8; body_len as usize];
        if body_len > 0 {
            self.read_binary(&mut body);
        }

        Ok(Message {
            message_type,
            flags,
            serial,
            member,
            interface,
            path,
            body: MessageParser::new(body),
        })
    }
}

fn main() {
    let mut dbus = Connection::new_session();
    dbus.authenticate();

    let hello_serial = dbus.send_hello();
    println!("Sent Hello with serial {}", hello_serial);

    let mut msg = dbus.read_message().unwrap();
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
