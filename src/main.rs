use std::os::{fd::AsRawFd, unix::net::UnixStream};

mod blocking_connection;
mod decoders;
mod encoders;
mod fsm;
mod io_uring_connection;
mod messages;
mod poll_connection;
mod serial;
mod types;

use crate::{
    blocking_connection::BlockingConnection,
    io_uring_connection::{IoUringAuth, IoUringConnection, IoUringConnector},
    messages::{NameAcquired, PropertiesChanged},
    poll_connection::PollConnection,
    types::{CompleteType, Message, Value},
};

fn session_connection() -> UnixStream {
    let address = std::env::var("DBUS_SESSION_BUS_ADDRESS").expect("no DBUS_SESSION_BUS_ADDRESS");
    let (_, path) = address.split_once("=").expect("no = separator");
    UnixStream::connect(path).expect("failed to create unix socket")
}

fn hello() -> Message {
    Message::MethodCall {
        serial: 0,
        path: String::from("/org/freedesktop/DBus"),
        member: String::from("Hello"),
        interface: Some(String::from("org.freedesktop.DBus")),
        destination: Some(String::from("org.freedesktop.DBus")),
        sender: None,
        unix_fds: None,
        body: vec![],
    }
}

fn show_notifiction() -> Message {
    Message::MethodCall {
        serial: 0,
        path: String::from("/org/freedesktop/Notifications"),
        member: String::from("Notify"),
        interface: Some(String::from("org.freedesktop.Notifications")),
        destination: Some(String::from("org.freedesktop.Notifications")),
        sender: None,
        unix_fds: None,
        body: vec![
            Value::String(String::from("")),
            Value::UInt32(1),
            Value::String(String::from("")),
            Value::String(String::from("Header")),
            Value::String(String::from("Body")),
            Value::Array(CompleteType::String, vec![]),
            Value::Array(
                CompleteType::DictEntry(
                    Box::new(CompleteType::String),
                    Box::new(CompleteType::Variant),
                ),
                vec![],
            ),
            Value::Int32(1_000),
        ],
    }
}

fn add_match(path: impl AsRef<str>) -> Message {
    Message::MethodCall {
        serial: 0,
        path: String::from("/org/freedesktop/DBus"),
        member: "AddMatch".to_string(),
        interface: Some(String::from("org.freedesktop.DBus")),
        destination: Some(String::from("org.freedesktop.DBus")),
        sender: None,
        unix_fds: None,
        body: vec![Value::String(format!(
            "type='signal',interface='org.freedesktop.DBus.Properties',member='PropertiesChanged',path='{}'",
            path.as_ref()
        ))],
    }
}

fn on_message(message: Message) {
    if let Ok(name_acquired) = NameAcquired::try_parse(&message) {
        println!("{name_acquired:?}");
    } else if let Ok(properties_changed) = PropertiesChanged::try_parse(&message) {
        println!("{properties_changed:?}");
    } else {
        println!("Unknown: {:?}", message);
    }
}

#[allow(dead_code)]
fn main_blocking() {
    let mut dbus = BlockingConnection::new(session_connection());
    let _guid = dbus.auth().unwrap();
    dbus.send_message(&mut hello()).unwrap();
    dbus.send_message(&mut show_notifiction()).unwrap();
    dbus.send_message(&mut add_match("/org/local/PipewireDBus"))
        .unwrap();

    loop {
        let message = dbus.read_message().unwrap();
        on_message(message);
    }
}

#[allow(dead_code)]
fn main_poll() {
    use libc::{POLLERR, POLLIN, POLLOUT, poll, pollfd};
    let mut dbus = PollConnection::new(session_connection());

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
    dbus.enqueue(&mut hello()).unwrap();
    dbus.enqueue(&mut show_notifiction()).unwrap();
    dbus.enqueue(&mut add_match("/org/local/PipewireDBus"))
        .unwrap();
    loop {
        fds[0].events = dbus.poll_read_write_events();
        let (readable, writable) = do_poll(&mut fds);

        if writable {
            dbus.poll_write_to_end().unwrap();
        }

        if readable {
            while let Some(message) = dbus.poll_read_one_message().unwrap() {
                on_message(message);
            }
        }
    }
}

#[allow(dead_code)]
fn main_io_uring() {
    use io_uring::IoUring;

    let mut ring = IoUring::new(10).unwrap();

    let mut connector = IoUringConnector::new();
    let fd = loop {
        let sqe = connector.next_sqe();
        unsafe { ring.submission().push(&sqe).unwrap() };

        let ready = ring.submit_and_wait(1).unwrap();
        assert_eq!(ready, 1);

        let cqe = ring.completion().next().expect("must be 1 item");
        if let Some(fd) = connector.process_cqe(cqe) {
            break fd;
        }
    };

    let mut auth = IoUringAuth::new(fd);
    loop {
        let sqe = auth.next_sqe();
        unsafe { ring.submission().push(&sqe).unwrap() };

        let ready = ring.submit_and_wait(1).unwrap();
        assert_eq!(ready, 1);

        let cqe = ring.completion().next().expect("must be 1 item");
        if let Some(guid) = auth.process_cqe(cqe) {
            println!("GUID: {guid:?}");
            break;
        }
    }

    let mut conn = IoUringConnection::new(fd);

    conn.enqueue(&mut hello()).unwrap();
    conn.enqueue(&mut show_notifiction()).unwrap();
    conn.enqueue(&mut add_match("/org/local/PipewireDBus"))
        .unwrap();

    loop {
        if let Some(sqe) = conn.next_sqe() {
            unsafe { ring.submission().push(&sqe).unwrap() };
        }

        ring.submit_and_wait(1).unwrap();
        ring.completion().sync();

        while let Some(cqe) = ring.completion().next() {
            if let Some(message) = conn.process_cqe(cqe) {
                on_message(message);
            }
        }
    }
}

fn main() {
    // main_blocking();
    // main_poll();
    main_io_uring();
}

#[test]
fn test_encode_decode_hello() {
    use crate::{decoders::MessageDecoder, encoders::MessageEncoder};
    let message = hello();
    let encoded = MessageEncoder::encode(&message).unwrap();
    let decoded = MessageDecoder::decode(&encoded).unwrap();
    assert_eq!(decoded, hello());
}

#[test]
fn test_encode_decode_show_notification() {
    use crate::{decoders::MessageDecoder, encoders::MessageEncoder};
    let message = show_notifiction();
    let encoded = MessageEncoder::encode(&message).unwrap();
    let decoded = MessageDecoder::decode(&encoded).unwrap();
    assert_eq!(decoded, show_notifiction());
}
