use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
};

const LITTLE_ENDIAN: u8 = b'l';
const MESSAGE_TYPE_METHOD_CALL: u8 = 1;
const MESSAGE_TYPE_METHOD_RETURN: u8 = 2;
const MESSAGE_TYPE_ERROR: u8 = 3;
const MESSAGE_TYPE_SIGNAL: u8 = 4;
const NO_REPLY_EXPECTED: u8 = 0x1;
const NO_AUTO_START: u8 = 0x2;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum HeaderField {
    Path = 1,
    Interface = 2,
    Member = 3,
    ErrorName = 4,
    ReplySerial = 5,
    Destination = 6,
    Sender = 7,
    Signature = 8,
}

impl TryFrom<u8> for HeaderField {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(HeaderField::Path),
            2 => Ok(HeaderField::Interface),
            3 => Ok(HeaderField::Member),
            4 => Ok(HeaderField::ErrorName),
            5 => Ok(HeaderField::ReplySerial),
            6 => Ok(HeaderField::Destination),
            7 => Ok(HeaderField::Sender),
            8 => Ok(HeaderField::Signature),
            _ => Err(value),
        }
    }
}

struct Message {
    msg_type: u8,
    flags: u8,
    serial: u32,
    member: Option<String>,
    interface: Option<String>,
    path: Option<String>,
    body: MessageBody,
}

struct MessageBody {
    data: Vec<u8>,
    pos: usize,
}

impl MessageBody {
    fn new(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }

    fn read_u32(&mut self) -> u32 {
        let value = u32::from_le_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ]);
        self.pos += 4;
        value
    }

    fn read_str(&mut self) -> &str {
        let len = self.read_u32() as usize;
        let s = std::str::from_utf8(&self.data[self.pos..self.pos + len])
            .expect("invalid UTF-8 in string");
        self.pos += len + 1; // +1 for null terminator
        s
    }
}

struct MessageBuilder {
    data: Vec<u8>,
}

impl MessageBuilder {
    fn new(msg_type: u8, flags: u8, body_length: u32) -> Self {
        let mut header = Vec::new();
        header.push(LITTLE_ENDIAN);
        header.push(msg_type);
        header.push(flags);
        header.push(1); // protocol version
        header.extend_from_slice(&body_length.to_le_bytes());
        header.extend_from_slice(&0u32.to_le_bytes()); // serial placeholder

        Self { data: header }
    }

    fn align_to(&mut self, alignment: usize) {
        // After finalize, there will be 16 bytes before our data (12 byte header + 4 byte array length)
        // So absolute position will be: 16 + (self.data.len() - 12)
        let fields_len = self.data.len() - 12;
        let absolute_pos = 16 + fields_len;
        let padding = (alignment - (absolute_pos % alignment)) % alignment;
        for _ in 0..padding {
            self.data.push(0);
        }
    }

    fn add_string_field(&mut self, field: HeaderField, value: &[u8]) {
        self.align_to(8);
        self.data.push(field as u8);
        self.data.push(1); // signature length
        self.data.push(b's');
        self.data.push(0); // null terminator for signature
        self.data
            .extend_from_slice(&(value.len() as u32).to_le_bytes());
        self.data.extend_from_slice(value);
        self.data.push(0); // null terminator for string
    }

    fn add_object_path_field(&mut self, field: HeaderField, value: &[u8]) {
        self.align_to(8);
        self.data.push(field as u8);
        self.data.push(1);
        self.data.push(b'o');
        self.data.push(0);
        self.data
            .extend_from_slice(&(value.len() as u32).to_le_bytes());
        self.data.extend_from_slice(value);
        self.data.push(0);
    }

    fn finalize(mut self, serial: u32) -> Vec<u8> {
        // Update the serial number
        self.data[8..12].copy_from_slice(&serial.to_le_bytes());

        // Calculate header fields array length (everything after the fixed 12-byte header)
        let header_fields_len = (self.data.len() - 12) as u32;

        // Insert the header fields array length at position 12
        let mut result = Vec::new();
        result.extend_from_slice(&self.data[..12]); // Fixed header
        result.extend_from_slice(&header_fields_len.to_le_bytes()); // Array length
        result.extend_from_slice(&self.data[12..]); // Header fields

        // Align to 8-byte boundary before body
        while result.len() % 8 != 0 {
            result.push(0);
        }

        result
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
        let mut msg = MessageBuilder::new(MESSAGE_TYPE_METHOD_CALL, 0, 0);
        msg.add_object_path_field(HeaderField::Path, b"/org/freedesktop/DBus");
        msg.add_string_field(HeaderField::Destination, b"org.freedesktop.DBus");
        msg.add_string_field(HeaderField::Interface, b"org.freedesktop.DBus");
        msg.add_string_field(HeaderField::Member, b"Hello");

        self.send_message(msg)
    }

    fn read_message(&mut self) -> Message {
        let mut header = [0u8; 16];
        self.read_exact(&mut header);

        let msg_type = header[1];
        let flags = header[2];
        let body_length = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
        let serial = u32::from_le_bytes([header[8], header[9], header[10], header[11]]);
        let header_fields_len =
            u32::from_le_bytes([header[12], header[13], header[14], header[15]]);

        let mut header_fields = vec![0u8; header_fields_len as usize];
        self.read_exact(&mut header_fields);

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
        let padding = (8 - ((16 + header_fields_len as usize) % 8)) % 8;
        if padding > 0 {
            let mut pad = [0u8; 7];
            self.read_exact(&mut pad[..padding]);
        }

        let mut body = vec![0u8; body_length as usize];
        if body_length > 0 {
            self.read_exact(&mut body);
        }

        Message {
            msg_type,
            flags,
            serial,
            member,
            interface,
            path,
            body: MessageBody::new(body),
        }
    }

    fn read_exact(&mut self, buf: &mut [u8]) {
        let mut total = 0;
        while total < buf.len() {
            let n = self.read_binary(&mut buf[total..]);
            total += n;
        }
    }
}

fn main() {
    let mut dbus = Connection::new_session();
    dbus.authenticate();

    let hello_serial = dbus.send_hello();
    println!("Sent Hello with serial {}", hello_serial);

    let mut msg = dbus.read_message();
    let unique_name = msg.body.read_str();
    println!("Our unique bus name: {}", unique_name);

    println!("\nWaiting for more messages...");
    loop {
        let mut msg = dbus.read_message();
        let msg_type_str = match msg.msg_type {
            MESSAGE_TYPE_METHOD_CALL => "METHOD_CALL",
            MESSAGE_TYPE_METHOD_RETURN => "METHOD_RETURN",
            MESSAGE_TYPE_ERROR => "ERROR",
            MESSAGE_TYPE_SIGNAL => "SIGNAL",
            _ => "UNKNOWN",
        };

        print!("Received {}", msg_type_str);
        if let Some(ref member) = msg.member {
            print!(" {}", member);
        }
        if let Some(ref interface) = msg.interface {
            print!(" (interface={})", interface);
        }
        println!(" serial={}, body_len={}", msg.serial, msg.body.data.len());

        if msg.msg_type == MESSAGE_TYPE_SIGNAL && msg.body.data.len() > 0 {
            let signal_arg = msg.body.read_str();
            println!("  Signal argument: {}", signal_arg);
        }

        return;
    }
}
