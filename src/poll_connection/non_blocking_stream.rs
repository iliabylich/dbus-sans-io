use anyhow::Result;
use std::{
    io::{ErrorKind, Read as _, Write as _},
    os::{fd::AsRawFd, unix::net::UnixStream},
};

pub(crate) struct NonBlockingUnixStream {
    s: UnixStream,
}

impl NonBlockingUnixStream {
    pub(crate) fn new(s: UnixStream) -> Self {
        Self { s }
    }

    pub(crate) fn read(&mut self, buf: &mut [u8]) -> Result<Option<usize>> {
        match self.s.read(buf) {
            Ok(len) => Ok(Some(len)),
            Err(err) if err.kind() == ErrorKind::WouldBlock => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    pub(crate) fn write(&mut self, buf: &[u8]) -> Result<Option<usize>> {
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
