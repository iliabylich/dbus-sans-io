use anyhow::Result;
use dbus_sans_io::{
    Message, Value, body_is, destination_is, interface_is, member_is, message_is,
    messages::{
        AddMatch, Hello, IntrospectRequest, IntrospectResponse, NameAcquired, PropertiesChanged,
        RequestName, ShowNotification,
    },
    path_is,
};
use std::borrow::Cow;

const INTROSPECTION: &str = r#"
<?xml version="1.0" encoding="UTF-8"?>
<node>
    <interface name="org.me.test">
        <method name="Plus">
            <arg type="i" name="x" direction="in" />
            <arg type="i" name="y" direction="in" />
            <arg type="i" name="sum" direction="out" />
        </method>
    </interface>
</node>
"#;

#[derive(Debug)]
struct PlusRequest {
    sender: Cow<'static, str>,
    serial: u32,
    lhs: i32,
    rhs: i32,
}

impl TryFrom<&Message> for PlusRequest {
    type Error = anyhow::Error;

    fn try_from(message: &Message) -> Result<Self> {
        message_is!(
            message,
            Message::MethodCall {
                serial,
                path,
                member,
                interface: Some(interface),
                destination: Some(destination),
                body,
                sender: Some(sender),
                ..
            }
        );

        path_is!(path, "/");
        member_is!(member, "Plus");
        interface_is!(interface, "org.me.test");
        destination_is!(destination, "org.me.test");
        body_is!(body, [Value::Int32(lhs), Value::Int32(rhs)]);

        Ok(Self {
            sender: sender.clone(),
            serial: *serial,
            lhs: *lhs,
            rhs: *rhs,
        })
    }
}

struct PlusResponse {
    req: PlusRequest,
}

impl PlusResponse {
    fn new(req: PlusRequest) -> Self {
        Self { req }
    }
}

impl From<PlusResponse> for Message {
    fn from(value: PlusResponse) -> Message {
        Message::MethodReturn {
            serial: 0,
            reply_serial: value.req.serial,
            destination: Some(value.req.sender),
            sender: None,
            unix_fds: None,
            body: vec![Value::Int32(value.req.lhs + value.req.rhs)],
        }
    }
}

fn on_message(message: Message) -> Vec<Message> {
    if let Ok(name_acquired) = NameAcquired::try_from(&message) {
        println!("{name_acquired:?}");
    } else if let Ok(properties_changed) = PropertiesChanged::try_from(&message) {
        println!("{properties_changed:?}");
    } else if let Ok(introspect_req) = IntrospectRequest::try_from(&message) {
        println!("{introspect_req:?}");
        if introspect_req.destination == "org.me.test" && introspect_req.path == "/" {
            let response = IntrospectResponse::new(introspect_req, INTROSPECTION).into();
            return vec![response];
        }
    } else if let Ok(plus_req) = PlusRequest::try_from(&message) {
        let response = PlusResponse::new(plus_req).into();
        return vec![response];
    } else {
        println!("Unknown: {:?}", message);
    }

    vec![]
}

#[cfg(feature = "blocking")]
fn main() -> Result<()> {
    println!("Blocking version\n\n");
    use dbus_sans_io::BlockingConnection;
    let mut conn = BlockingConnection::session()?;

    conn.auth()?;
    conn.send_message(&mut Hello.into())?;
    conn.send_message(&mut ShowNotification::new("Header", "Body").into())?;
    conn.send_message(&mut AddMatch::new(Cow::Borrowed("/org/local/PipewireDBus")).into())?;
    conn.send_message(&mut RequestName::new(Cow::Borrowed("org.me.test")).into())?;

    loop {
        let message = conn.read_message()?;
        for mut reply in on_message(message) {
            conn.send_message(&mut reply)?;
        }
    }
}

#[cfg(feature = "poll")]
fn main() -> Result<()> {
    println!("Poll version\n\n");

    use dbus_sans_io::PollConnection;
    use libc::{POLLERR, POLLIN, POLLOUT, poll, pollfd};
    use std::os::fd::AsRawFd;
    let mut conn = PollConnection::session()?;

    let mut fds = [pollfd {
        fd: conn.as_raw_fd(),
        events: POLLIN | POLLOUT,
        revents: 0,
    }];

    fn do_poll(fds: &mut [pollfd; 1]) -> (bool, bool) {
        let res = unsafe { poll(fds.as_mut_ptr(), 1, -1) };
        assert!(res == 1);
        let readable = fds[0].revents & POLLIN != 0;
        let writable = fds[0].revents & POLLOUT != 0;
        assert_eq!(fds[0].revents & POLLERR, 0);
        (readable, writable)
    }

    conn.enqueue(&mut Hello.into())?;
    conn.enqueue(&mut ShowNotification::new("Header", "Body").into())?;
    conn.enqueue(&mut AddMatch::new(Cow::Borrowed("/org/local/PipewireDBus")).into())?;
    conn.enqueue(&mut RequestName::new(Cow::Borrowed("org.me.test")).into())?;

    loop {
        fds[0].events = conn.events();
        let (readable, writable) = do_poll(&mut fds);

        for message in conn.poll(readable, writable)? {
            for mut reply in on_message(message) {
                conn.enqueue(&mut reply)?;
            }
        }
    }
}

#[cfg(feature = "io-uring")]
fn main() -> Result<()> {
    println!("io_uring version\n\n");

    use io_uring::IoUring;
    let mut ring = IoUring::new(10)?;

    use dbus_sans_io::IoUringConnection;
    let mut conn = IoUringConnection::session();

    conn.enqueue(&mut Hello.into())?;
    conn.enqueue(&mut ShowNotification::new("Header", "Body").into())?;
    conn.enqueue(&mut AddMatch::new(Cow::Borrowed("/org/local/PipewireDBus")).into())?;
    conn.enqueue(&mut RequestName::new(Cow::Borrowed("org.me.test")).into())?;

    loop {
        if let Some(sqe) = conn.next_sqe() {
            unsafe { ring.submission().push(&sqe)? };
        }

        ring.submit_and_wait(1)?;

        while let Some(cqe) = ring.completion().next() {
            if let Some(message) = conn.process_cqe(cqe)? {
                let replies = on_message(message);
                for mut reply in replies {
                    println!("Replying with {reply:?}");
                    conn.enqueue(&mut reply)?;
                }
            }
        }
    }
}
