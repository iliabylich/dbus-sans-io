use libc::{POLLERR, POLLIN, POLLOUT, poll, pollfd};
use std::os::{fd::AsRawFd, unix::net::UnixStream};

mod blocking_connection;
mod decoders;
mod encoders;
mod fsm;
mod poll_connection;
mod serial;
mod types;

use crate::{
    blocking_connection::BlockingConnection,
    poll_connection::PollConnection,
    types::{Flags, Header, Message, MessageType, ObjectPath},
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
    let mut dbus = BlockingConnection::new(session_connection());
    let _guid = dbus.auth().unwrap();
    let _serial = dbus.send_message(hello()).unwrap();
    loop {
        let msg = dbus.read_message().unwrap();

        println!("Received {:?}", msg);
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
    let _sent = dbus.enqueue(hello()).unwrap();
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
