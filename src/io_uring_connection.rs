use crate::{
    fsm::{AuthFSM, AuthWants, ConnectFSM, ConnectWants, ReaderFSM, WriterFSM},
    serial::Serial,
    types::{GUID, Message},
};
use anyhow::Result;
use io_uring::{cqueue::Entry as Cqe, opcode, squeue::Entry as Sqe, types};
use libc::{AF_UNIX, SOCK_STREAM, sockaddr, sockaddr_un};

const SOCKET_USER_DATA: u64 = 1;
const CONNECT_USER_DATA: u64 = 2;
const READ_USER_DATA: u64 = 3;
const WRITE_USER_DATA: u64 = 4;

#[derive(Debug)]
pub(crate) struct IoUringConnector {
    connect: ConnectFSM,
}

impl IoUringConnector {
    pub(crate) fn new() -> Self {
        Self {
            connect: ConnectFSM::new(),
        }
    }

    pub(crate) fn next_sqe(&mut self) -> Sqe {
        match self
            .connect
            .wants()
            .expect("IoUringConnector is Done, did you forget to feed it with last CQE entry?")
        {
            ConnectWants::Socket => socket_sqe(),
            ConnectWants::Connect { fd, addr } => connect_sqe(fd, addr),
        }
    }

    pub(crate) fn process_cqe(&mut self, cqe: Cqe) -> Option<i32> {
        match cqe.user_data() {
            SOCKET_USER_DATA => {
                let fd = cqe.result();
                assert!(fd > 0);
                self.connect.satisfy_socket(fd);
            }

            CONNECT_USER_DATA => {
                assert!(cqe.result() >= 0);
                self.connect.satisfy_connect();
                return self.connect.fd();
            }

            _ => {}
        }

        None
    }
}

pub(crate) struct IoUringAuth {
    fd: i32,
    auth: AuthFSM,
}

impl IoUringAuth {
    pub(crate) fn new(fd: i32) -> Self {
        Self {
            fd,
            auth: AuthFSM::new(),
        }
    }

    pub(crate) fn next_sqe(&mut self) -> Sqe {
        match self.auth.wants() {
            AuthWants::Read(buf) => read_sqe(self.fd, buf),
            AuthWants::Write(buf) => write_sqe(self.fd, buf),
        }
    }

    pub(crate) fn process_cqe(&mut self, cqe: Cqe) -> Option<GUID> {
        match cqe.user_data() {
            WRITE_USER_DATA => {
                let written = cqe.result();
                assert!(written >= 0);
                let written = written as usize;

                if let Some(guid) = self.auth.satisfy_write(written).unwrap() {
                    return Some(guid);
                }
            }

            READ_USER_DATA => {
                let read = cqe.result();
                assert!(read >= 0);
                let read = read as usize;

                self.auth.satisfy_read(read).unwrap();
            }

            _ => {}
        }

        None
    }
}

#[derive(Debug)]
pub(crate) struct IoUringConnection {
    fd: i32,
    serial: Serial,
    reader: BufferedReaderFSM,
    writer: WriterFSM,
}

#[derive(Debug)]
struct BufferedReaderFSM {
    reader: ReaderFSM,
    last: *mut u8,
}
impl BufferedReaderFSM {
    fn new() -> Self {
        Self {
            reader: ReaderFSM::new(),
            last: std::ptr::null_mut(),
        }
    }

    fn wants(&mut self) -> Option<&mut [u8]> {
        let new = self.reader.wants();
        if new.as_ptr() != self.last {
            self.last = new.as_mut_ptr();
            Some(new)
        } else {
            return None;
        }
    }

    fn satisfy(&mut self, read: usize) -> Result<Option<Message>> {
        self.reader.satisfy(read)
    }
}

impl IoUringConnection {
    pub(crate) fn new(fd: i32) -> Self {
        Self {
            fd,
            serial: Serial::zero(),
            reader: BufferedReaderFSM::new(),
            writer: WriterFSM::new(),
        }
    }

    pub(crate) fn enqueue(&mut self, message: &mut Message) -> Result<()> {
        *message.serial_mut() = self.serial.increment_and_get();
        self.writer.enqueue(message)
    }

    pub(crate) fn next_sqe(&mut self) -> Option<Sqe> {
        if let Some(buf) = self.writer.wants() {
            return Some(write_sqe(self.fd, buf));
        }

        if let Some(buf) = self.reader.wants() {
            return Some(read_sqe(self.fd, buf));
        }

        None
    }

    pub(crate) fn process_cqe(&mut self, cqe: Cqe) -> Option<Message> {
        match cqe.user_data() {
            WRITE_USER_DATA => {
                let written = cqe.result();
                assert!(written >= 0);
                let written = written as usize;

                self.writer.satisfy(written).unwrap();
            }

            READ_USER_DATA => {
                let read = cqe.result();
                assert!(read >= 0);
                let read = read as usize;

                if let Some(message) = self.reader.satisfy(read).unwrap() {
                    return Some(message);
                }
            }

            _ => {}
        }

        None
    }
}

fn socket_sqe() -> Sqe {
    opcode::Socket::new(AF_UNIX, SOCK_STREAM, 0)
        .build()
        .user_data(SOCKET_USER_DATA)
}

fn connect_sqe(fd: i32, addr: *const sockaddr_un) -> Sqe {
    opcode::Connect::new(
        types::Fd(fd),
        addr.cast::<sockaddr>(),
        std::mem::size_of::<sockaddr_un>() as u32,
    )
    .build()
    .user_data(CONNECT_USER_DATA)
}

fn write_sqe(fd: i32, buf: &[u8]) -> Sqe {
    opcode::Write::new(types::Fd(fd), buf.as_ptr(), buf.len() as u32)
        .build()
        .user_data(WRITE_USER_DATA)
}

fn read_sqe(fd: i32, buf: &mut [u8]) -> Sqe {
    opcode::Read::new(types::Fd(fd), buf.as_mut_ptr(), buf.len() as u32)
        .build()
        .user_data(READ_USER_DATA)
}
