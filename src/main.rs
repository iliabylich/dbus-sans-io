use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
};

struct Connection {
    stream: UnixStream,
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

        Self { stream }
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

        let mut buf = [0; 1_024];
        let len = self.read_binary(&mut buf);
        assert_eq!(&buf[..len], b"DATA\r\n");

        self.write_all(b"DATA\r\n");

        let mut buf = [0; 1_024];
        let len = self.read_binary(&mut buf);
        let guid = buf[..len]
            .strip_prefix(b"OK ")
            .expect("no OK prefix in data response")
            .strip_suffix(b"\r\n")
            .expect("no \\r\\n suffix in data response");
        println!("GUID: {:?}", std::str::from_utf8(guid));

        self.write_all(b"BEGIN\r\n");
    }
}

fn main() {
    let mut dbus = Connection::new_session();
    dbus.authenticate();
}
