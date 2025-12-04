use crate::{
    encoders::MessageEncoder,
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
    serial: Serial,
    queue: Vec<Vec<u8>>,
}

impl IoUringConnector {
    pub(crate) fn new() -> Self {
        Self {
            connect: ConnectFSM::new(),
            serial: Serial::zero(),
            queue: vec![],
        }
    }

    pub(crate) fn enqueue(&mut self, message: &mut Message) -> Result<()> {
        *message.serial_mut() = self.serial.increment_and_get();
        let buf = MessageEncoder::encode(message)?;
        self.queue.push(buf);
        Ok(())
    }

    pub(crate) fn next_sqe(&mut self) -> Sqe {
        match self.connect.wants() {
            ConnectWants::Socket => socket_sqe(),
            ConnectWants::Connect { fd, addr } => connect_sqe(fd, addr),
        }
    }

    pub(crate) fn process_cqe(&mut self, cqe: Cqe) -> Result<Option<i32>> {
        match cqe.user_data() {
            SOCKET_USER_DATA => {
                let fd = cqe.result();
                assert!(fd > 0);
                self.connect.satisfy_socket(fd)?;
                Ok(None)
            }

            CONNECT_USER_DATA => {
                assert!(cqe.result() >= 0);
                let fd = self.connect.satisfy_connect();
                Ok(Some(fd))
            }

            _ => Ok(None),
        }
    }
}

pub(crate) struct IoUringAuth {
    fd: i32,
    serial: Serial,
    queue: Vec<Vec<u8>>,
    auth: AuthFSM,
}

impl IoUringAuth {
    pub(crate) fn new(fd: i32, serial: Serial, queue: Vec<Vec<u8>>) -> Self {
        Self {
            fd,
            serial,
            queue,
            auth: AuthFSM::new(),
        }
    }

    pub(crate) fn enqueue(&mut self, message: &mut Message) -> Result<()> {
        *message.serial_mut() = self.serial.increment_and_get();
        let buf = MessageEncoder::encode(message)?;
        self.queue.push(buf);
        Ok(())
    }

    pub(crate) fn next_sqe(&mut self) -> Sqe {
        match self.auth.wants() {
            AuthWants::Read(buf) => read_sqe(self.fd, buf),
            AuthWants::Write(buf) => write_sqe(self.fd, buf),
        }
    }

    pub(crate) fn process_cqe(&mut self, cqe: Cqe) -> Result<Option<GUID>> {
        match cqe.user_data() {
            WRITE_USER_DATA => {
                let written = cqe.result();
                assert!(written >= 0);
                let written = written as usize;

                if let Some(guid) = self.auth.satisfy_write(written)? {
                    return Ok(Some(guid));
                }
                Ok(None)
            }

            READ_USER_DATA => {
                let read = cqe.result();
                assert!(read >= 0);
                let read = read as usize;

                self.auth.satisfy_read(read)?;
                Ok(None)
            }

            _ => Ok(None),
        }
    }
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

#[derive(Debug)]
pub(crate) struct IoUringReaderWriter {
    fd: i32,
    serial: Serial,
    reader: BufferedReaderFSM,
    writer: WriterFSM,
}

impl IoUringReaderWriter {
    pub(crate) fn new(fd: i32, serial: Serial, queue: Vec<Vec<u8>>) -> Self {
        let mut writer = WriterFSM::new();
        for buf in queue {
            writer.enqueue_serialized(buf);
        }

        Self {
            fd,
            serial,
            reader: BufferedReaderFSM::new(),
            writer,
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

    pub(crate) fn process_cqe(&mut self, cqe: Cqe) -> Result<Option<Message>> {
        match cqe.user_data() {
            WRITE_USER_DATA => {
                let written = cqe.result();
                assert!(written >= 0);
                let written = written as usize;

                self.writer.satisfy(written)?;
                Ok(None)
            }

            READ_USER_DATA => {
                let read = cqe.result();
                assert!(read >= 0);
                let read = read as usize;

                if let Some(message) = self.reader.satisfy(read)? {
                    return Ok(Some(message));
                }
                Ok(None)
            }

            _ => Ok(None),
        }
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

#[derive(Default)]
pub(crate) enum IoUringConnection {
    #[default]
    None,

    Connecting(IoUringConnector),
    Auth(IoUringAuth),
    ReaderWriter(IoUringReaderWriter),
}

impl IoUringConnection {
    pub(crate) fn new() -> Self {
        Self::Connecting(IoUringConnector::new())
    }

    pub(crate) fn enqueue(&mut self, message: &mut Message) -> Result<()> {
        match self {
            Self::Connecting(connector) => connector.enqueue(message),
            Self::Auth(auth) => auth.enqueue(message),
            Self::ReaderWriter(rw) => rw.enqueue(message),

            Self::None => unreachable!(),
        }
    }

    pub(crate) fn next_sqe(&mut self) -> Option<Sqe> {
        match self {
            Self::Connecting(connector) => Some(connector.next_sqe()),
            Self::Auth(auth) => Some(auth.next_sqe()),
            Self::ReaderWriter(reader_writer) => reader_writer.next_sqe(),

            Self::None => unreachable!(),
        }
    }

    pub(crate) fn process_cqe(&mut self, cqe: Cqe) -> Result<Option<Message>> {
        match self {
            Self::Connecting(connector) => match connector.process_cqe(cqe)? {
                Some(fd) => {
                    let Self::Connecting(IoUringConnector { serial, queue, .. }) =
                        std::mem::take(self)
                    else {
                        unreachable!()
                    };

                    *self = Self::Auth(IoUringAuth::new(fd, serial, queue));
                    Ok(None)
                }
                None => Ok(None),
            },

            Self::Auth(auth) => match auth.process_cqe(cqe)? {
                Some(guid) => {
                    println!("GUID: {guid:?}");
                    let Self::Auth(IoUringAuth {
                        fd, serial, queue, ..
                    }) = std::mem::take(self)
                    else {
                        unreachable!()
                    };
                    *self = Self::ReaderWriter(IoUringReaderWriter::new(fd, serial, queue));
                    Ok(None)
                }
                None => Ok(None),
            },

            Self::ReaderWriter(reader_writer) => reader_writer.process_cqe(cqe),

            Self::None => unreachable!(),
        }
    }
}
