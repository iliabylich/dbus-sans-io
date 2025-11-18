use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
};

const LITTLE_ENDIAN: u8 = b'l';
const MESSAGE_TYPE_METHOD_CALL: u8 = 1;
const NO_REPLY_EXPECTED: u8 = 0x1;
const NO_AUTO_START: u8 = 0x2;

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

    fn add_string_field(&mut self, field_code: u8, value: &[u8]) {
        self.align_to(8);
        self.data.push(field_code);
        self.data.push(1); // signature length
        self.data.push(b's');
        self.data.push(0); // null terminator for signature
        self.data
            .extend_from_slice(&(value.len() as u32).to_le_bytes());
        self.data.extend_from_slice(value);
        self.data.push(0); // null terminator for string
    }

    fn add_object_path_field(&mut self, field_code: u8, value: &[u8]) {
        self.align_to(8);
        self.data.push(field_code);
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
        msg.add_object_path_field(1, b"/org/freedesktop/DBus");
        msg.add_string_field(6, b"org.freedesktop.DBus"); // DESTINATION
        msg.add_string_field(2, b"org.freedesktop.DBus"); // INTERFACE
        msg.add_string_field(3, b"Hello"); // MEMBER

        self.send_message(msg)
    }

    fn read_message(&mut self) -> MessageBody {
        let mut header = [0u8; 16];
        self.read_exact(&mut header);

        let body_length = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
        let serial = u32::from_le_bytes([header[8], header[9], header[10], header[11]]);
        let header_fields_len =
            u32::from_le_bytes([header[12], header[13], header[14], header[15]]);

        println!(
            "Message: type={}, flags={}, body_len={}, serial={}, header_fields_len={}",
            header[1], header[2], body_length, serial, header_fields_len
        );

        let mut header_fields = vec![0u8; header_fields_len as usize];
        self.read_exact(&mut header_fields);

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

        MessageBody::new(body)
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

    let mut body = dbus.read_message();
    let unique_name = body.read_str();
    println!("Our unique bus name: {}", unique_name);
}
