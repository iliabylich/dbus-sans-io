use anyhow::Result;
use libc::{POLLIN, POLLOUT};
use std::{
    io::{ErrorKind, Read as _, Write as _},
    os::{fd::AsRawFd, unix::net::UnixStream},
};

use crate::{
    fsm::{AuthFSM, AuthWants, ReaderFSM, WriterFSM},
    serial::Serial,
    types::{GUID, Message},
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

pub(crate) struct PollConnection {
    stream: NonBlockingUnixStream,
    serial: Serial,

    auth: AuthFSM,
    reader: ReaderFSM,
    writer: WriterFSM,
}

impl AsRawFd for PollConnection {
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        self.stream.as_raw_fd()
    }
}

impl PollConnection {
    pub(crate) fn new(stream: UnixStream) -> Self {
        stream.set_nonblocking(true).unwrap();

        Self {
            stream: NonBlockingUnixStream::new(stream),
            serial: Serial::zero(),

            auth: AuthFSM::new(),
            reader: ReaderFSM::new(),
            writer: WriterFSM::new(),
        }
    }

    pub(crate) fn enqueue(&mut self, message: &mut Message) -> Result<()> {
        *message.serial_mut() = self.serial.increment_and_get();

        self.writer.enqueue(message)?;
        Ok(())
    }

    pub(crate) fn poll_auth_events(&mut self) -> i16 {
        match self.auth.wants() {
            AuthWants::Read(_) => POLLIN,
            AuthWants::Write(_) => POLLOUT,
        }
    }

    pub(crate) fn poll_auth(&mut self, readable: bool, writable: bool) -> Result<Option<GUID>> {
        loop {
            let mut did = false;

            if writable
                && let AuthWants::Write(buf) = self.auth.wants()
                && let Some(len) = self.stream.write(buf)?
            {
                did = true;
                if let Some(guid) = self.auth.satisfy_write(len)? {
                    return Ok(Some(guid));
                }
            }

            if readable
                && let AuthWants::Read(buf) = self.auth.wants()
                && let Some(len) = self.stream.read(buf)?
            {
                did = true;
                self.auth.satisfy_read(len)?;
            }

            if !did {
                break;
            }
        }

        Ok(None)
    }

    pub(crate) fn poll_read_write_events(&mut self) -> i16 {
        let mut out = POLLIN;
        if self.writer.wants().is_some() {
            out |= POLLOUT;
        }
        out
    }

    pub(crate) fn poll_read_one_message(&mut self) -> Result<Option<Message>> {
        loop {
            let buf = self.reader.wants();
            let Some(len) = self.stream.read(buf)? else {
                return Ok(None);
            };

            if let Some(message) = self.reader.satisfy(len)? {
                return Ok(Some(message));
            }
        }
    }

    pub(crate) fn poll_write_to_end(&mut self) -> Result<()> {
        loop {
            let Some(buf) = self.writer.wants() else {
                break;
            };
            let Some(len) = self.stream.write(buf)? else {
                break;
            };
            self.writer.satisfy(len)?;
        }
        Ok(())
    }
}
