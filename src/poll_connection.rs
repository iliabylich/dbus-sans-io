use crate::{
    encoders::MessageEncoder,
    fsm::{AuthFSM, AuthWants, AuthWantsTag, ReaderFSM, WriterFSM},
    serial::Serial,
    types::Message,
};
use anyhow::Result;
use libc::{POLLIN, POLLOUT};
use std::{
    io::{ErrorKind, Read as _, Write as _},
    os::{fd::AsRawFd, unix::net::UnixStream},
};

struct NonBlockingUnixStream {
    s: UnixStream,
}

impl NonBlockingUnixStream {
    fn new(s: UnixStream) -> Self {
        Self { s }
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<Option<usize>> {
        match self.s.read(buf) {
            Ok(len) => Ok(Some(len)),
            Err(err) if err.kind() == ErrorKind::WouldBlock => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    fn write(&mut self, buf: &[u8]) -> Result<Option<usize>> {
        match self.s.write(buf) {
            Ok(len) => Ok(Some(len)),
            Err(err) if err.kind() == ErrorKind::WouldBlock => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}

impl AsRawFd for NonBlockingUnixStream {
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        self.s.as_raw_fd()
    }
}

pub(crate) struct PollAuthFSM {
    stream: NonBlockingUnixStream,
    serial: Serial,
    auth: AuthFSM,
    queue: Vec<Vec<u8>>,
}

impl PollAuthFSM {
    fn new(stream: NonBlockingUnixStream) -> Self {
        Self {
            stream,
            serial: Serial::zero(),
            auth: AuthFSM::new(),
            queue: Vec::new(),
        }
    }

    fn enqueue(&mut self, message: &mut Message) -> Result<()> {
        *message.serial_mut() = self.serial.increment_and_get();
        let buf = MessageEncoder::encode(message)?;
        self.queue.push(buf);
        Ok(())
    }

    fn events(&self) -> i16 {
        match self.auth.wants_tag() {
            AuthWantsTag::Read => POLLIN,
            AuthWantsTag::Write => POLLOUT,
        }
    }

    fn poll(&mut self, readable: bool, writable: bool) -> Result<bool> {
        loop {
            let mut did = false;

            if writable
                && let AuthWants::Write(buf) = self.auth.wants()
                && let Some(written) = self.stream.write(buf)?
            {
                did = true;
                if let Some(guid) = self.auth.satisfy_write(written)? {
                    println!("GUID: {guid:?}");
                    return Ok(true);
                }
            }

            if readable
                && let AuthWants::Read(buf) = self.auth.wants()
                && let Some(read) = self.stream.read(buf)?
            {
                did = true;
                self.auth.satisfy_read(read)?;
            }

            if !did {
                break;
            }
        }

        Ok(false)
    }
}

pub(crate) struct PollReaderWriterFSM {
    stream: NonBlockingUnixStream,
    serial: Serial,
    reader: ReaderFSM,
    writer: WriterFSM,
}

impl PollReaderWriterFSM {
    fn new(stream: NonBlockingUnixStream, serial: Serial, queue: Vec<Vec<u8>>) -> Self {
        let mut writer = WriterFSM::new();
        for buf in queue {
            writer.enqueue_serialized(buf);
        }
        Self {
            stream,
            serial,
            reader: ReaderFSM::new(),
            writer,
        }
    }

    fn enqueue(&mut self, message: &mut Message) -> Result<()> {
        *message.serial_mut() = self.serial.increment_and_get();
        self.writer.enqueue(message)?;
        Ok(())
    }

    fn events(&self) -> i16 {
        let mut out = POLLIN;
        if self.writer.wants().is_some() {
            out |= POLLOUT;
        }
        out
    }

    fn poll(&mut self, readable: bool, writable: bool) -> Result<Vec<Message>> {
        if writable {
            loop {
                let Some(buf) = self.writer.wants() else {
                    break;
                };
                let Some(len) = self.stream.write(buf)? else {
                    break;
                };
                self.writer.satisfy(len)?;
            }
        }

        if readable {
            let mut messages = vec![];

            loop {
                let buf = self.reader.wants();
                let Some(len) = self.stream.read(buf)? else {
                    return Ok(messages);
                };

                if let Some(message) = self.reader.satisfy(len)? {
                    messages.push(message);
                }
            }
        }

        Ok(vec![])
    }
}

#[derive(Default)]
pub(crate) enum PollConnection {
    #[default]
    None,
    Auth(PollAuthFSM),
    ReaderWriter(PollReaderWriterFSM),
}

impl AsRawFd for PollConnection {
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        match self {
            Self::Auth(PollAuthFSM { stream, .. })
            | Self::ReaderWriter(PollReaderWriterFSM { stream, .. }) => stream.as_raw_fd(),

            Self::None => unreachable!(),
        }
    }
}

impl PollConnection {
    pub(crate) fn new(stream: UnixStream) -> Self {
        stream.set_nonblocking(true).unwrap();

        Self::Auth(PollAuthFSM::new(NonBlockingUnixStream::new(stream)))
    }

    pub(crate) fn enqueue(&mut self, message: &mut Message) -> Result<()> {
        match self {
            Self::Auth(auth) => auth.enqueue(message),
            Self::ReaderWriter(rw) => rw.enqueue(message),

            Self::None => unreachable!(),
        }
    }

    pub(crate) fn events(&self) -> i16 {
        match self {
            Self::Auth(auth) => auth.events(),
            Self::ReaderWriter(rw) => rw.events(),

            Self::None => unreachable!(),
        }
    }

    pub(crate) fn poll(&mut self, readable: bool, writable: bool) -> Result<Vec<Message>> {
        match self {
            Self::Auth(auth) => {
                if auth.poll(readable, writable)? {
                    // EOA
                    let Self::Auth(PollAuthFSM {
                        stream,
                        serial,
                        queue,
                        ..
                    }) = std::mem::take(self)
                    else {
                        unreachable!()
                    };

                    *self = Self::ReaderWriter(PollReaderWriterFSM::new(stream, serial, queue));
                }

                Ok(vec![])
            }
            Self::ReaderWriter(rw) => rw.poll(readable, writable),

            Self::None => unreachable!(),
        }
    }
}
