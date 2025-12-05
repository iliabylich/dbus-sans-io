use crate::{
    Message,
    fsm::{ReaderFSM, WriterFSM},
    poll_connection::non_blocking_stream::NonBlockingUnixStream,
};
use anyhow::Result;
use libc::{POLLIN, POLLOUT};
use std::os::fd::AsRawFd;

pub(crate) struct PollReaderWriterFSM {
    stream: NonBlockingUnixStream,
    reader: ReaderFSM,
    writer: WriterFSM,
}

impl PollReaderWriterFSM {
    pub(crate) fn new(stream: NonBlockingUnixStream, queue: Vec<Vec<u8>>) -> Self {
        let mut writer = WriterFSM::new();
        for buf in queue {
            writer.enqueue(buf);
        }
        Self {
            stream,
            reader: ReaderFSM::new(),
            writer,
        }
    }

    pub(crate) fn enqueue(&mut self, buf: Vec<u8>) {
        self.writer.enqueue(buf);
    }

    pub(crate) fn events(&self) -> i16 {
        let mut out = POLLIN;
        if self.writer.wants().is_some() {
            out |= POLLOUT;
        }
        out
    }

    pub(crate) fn poll(&mut self, readable: bool, writable: bool) -> Result<Vec<Message>> {
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

impl AsRawFd for PollReaderWriterFSM {
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        self.stream.as_raw_fd()
    }
}
