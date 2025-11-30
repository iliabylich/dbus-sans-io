use libc::{POLLERR, POLLIN, POLLOUT, poll, pollfd};
use std::os::{fd::AsRawFd, unix::net::UnixStream};

mod blocking_connection;
mod decoders;
mod encoders;
mod fsm;
mod messages;
mod poll_connection;
mod serial;
mod types;

use crate::{
    blocking_connection::BlockingConnection,
    decoders::MessageDecoder,
    encoders::MessageEncoder,
    messages::NameAcquired,
    poll_connection::PollConnection,
    types::{CompleteType, Flags, Header, Message, MessageType, Value},
};

fn session_connection() -> UnixStream {
    let address = std::env::var("DBUS_SESSION_BUS_ADDRESS").expect("no DBUS_SESSION_BUS_ADDRESS");
    let (_, path) = address.split_once("=").expect("no = separator");
    UnixStream::connect(path).expect("failed to create unix socket")
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
        path: Some(b"/org/freedesktop/DBus".to_vec()),
        error_name: None,
        reply_serial: None,
        destination: Some(String::from("org.freedesktop.DBus")),
        sender: None,
        signature: None,
        unix_fds: None,
        body: vec![],
    }
}

fn show_notifiction(sender: &str) -> Message {
    Message {
        header: Header {
            message_type: MessageType::MethodCall,
            flags: Flags { byte: 0 },
            serial: 0,
            body_len: 0,
        },
        member: Some(String::from("Notify")),
        interface: Some(String::from("org.freedesktop.Notifications")),
        path: Some(b"/org/freedesktop/Notifications".to_vec()),
        error_name: None,
        reply_serial: None,
        destination: Some(String::from("org.freedesktop.Notifications")),
        sender: Some(sender.to_string()),
        signature: None,
        unix_fds: None,
        body: vec![
            Value::String(String::from("")),
            Value::UInt32(1),
            Value::String(String::from("")),
            Value::String(String::from("Header")),
            Value::String(String::from("Body")),
            Value::Array(CompleteType::Struct(vec![CompleteType::String]), vec![]),
            Value::Array(
                CompleteType::Struct(vec![CompleteType::String, CompleteType::Variant]),
                vec![],
            ),
            Value::Int32(1_000),
        ],
    }
    .with_generated_signature()
}

#[allow(dead_code)]
fn main_blocking() {
    // println!("{:?}", show_notifiction());
    // println!("{:?}", show_notifiction().with_generated_signature());
    let mut dbus = BlockingConnection::new(session_connection());
    let _guid = dbus.auth().unwrap();
    dbus.send_message(&mut hello()).unwrap();

    loop {
        let message = dbus.read_message().unwrap();

        match NameAcquired::try_from(message) {
            Ok(name_acquired) => {
                println!("{name_acquired:?}");
                let sender = name_acquired.name;
                dbus.send_message(&mut show_notifiction(&sender)).unwrap();
                println!("notification sent");
            }
            Err(message) => {
                println!("Received unknown {:?}", message);
            }
        }
    }
}

#[allow(dead_code)]
fn main_poll() {
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
    loop {
        fds[0].events = dbus.poll_read_write_events();
        let (readable, writable) = do_poll(&mut fds);

        if writable {
            dbus.poll_write_to_end().unwrap();
        }

        if readable {
            while let Some(message) = dbus.poll_read_one_message().unwrap() {
                match NameAcquired::try_from(message) {
                    Ok(name_acquired) => {
                        println!("{name_acquired:?}");
                        let sender = name_acquired.name;
                        dbus.enqueue(&mut show_notifiction(&sender)).unwrap();
                    }
                    Err(message) => {
                        println!("Unknown: {:?}", message);
                    }
                }
            }
        }
    }
}

fn main() {
    main_blocking();
    // main_poll();
}

#[test]
fn test_encode_decode_hello() {
    let message = hello();
    let encoded = MessageEncoder::encode(&message).unwrap();
    let decoded = MessageDecoder::decode(&encoded).unwrap();
    assert_eq!(message, decoded);
}

#[test]
fn test_encode_decode_show_notification() {
    let message = show_notifiction(":1.100");
    let encoded = MessageEncoder::encode(&message).unwrap();
    let decoded = MessageDecoder::decode(&encoded).unwrap();
    assert_eq!(message, decoded);
}
