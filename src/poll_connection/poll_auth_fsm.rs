use std::os::fd::AsRawFd;

use crate::{
    fsm::{AuthFSM, AuthWants, AuthWantsTag},
    poll_connection::non_blocking_stream::NonBlockingUnixStream,
};
use anyhow::Result;
use libc::{POLLIN, POLLOUT};

pub(crate) struct PollAuthFSM {
    pub(crate) stream: NonBlockingUnixStream,
    pub(crate) queue: Vec<Vec<u8>>,
    auth: AuthFSM,
}

impl PollAuthFSM {
    pub(crate) fn new(stream: NonBlockingUnixStream) -> Self {
        Self {
            stream,
            auth: AuthFSM::new(),
            queue: Vec::new(),
        }
    }

    pub(crate) fn enqueue(&mut self, buf: Vec<u8>) {
        self.queue.push(buf);
    }

    pub(crate) fn events(&self) -> i16 {
        match self.auth.wants_tag() {
            AuthWantsTag::Read => POLLIN,
            AuthWantsTag::Write => POLLOUT,
        }
    }

    pub(crate) fn poll(&mut self, readable: bool, writable: bool) -> Result<bool> {
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

impl AsRawFd for PollAuthFSM {
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        self.stream.as_raw_fd()
    }
}
