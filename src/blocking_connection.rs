use crate::{
    fsm::{AuthFSM, AuthWants, ReaderFSM, WriterFSM},
    serial::Serial,
    types::{GUID, Message},
};
use anyhow::Result;
use std::{
    io::{Read as _, Write as _},
    os::unix::net::UnixStream,
};

pub(crate) struct BlockingConnection {
    stream: UnixStream,
    serial: Serial,

    auth: AuthFSM,
    reader: ReaderFSM,
    writer: WriterFSM,
}

impl BlockingConnection {
    pub(crate) fn new(stream: UnixStream) -> Self {
        Self {
            stream,
            serial: Serial::zero(),

            auth: AuthFSM::new(),
            reader: ReaderFSM::new(),
            writer: WriterFSM::new(),
        }
    }

    pub(crate) fn auth(&mut self) -> Result<GUID> {
        loop {
            match self.auth.wants() {
                AuthWants::Read(buf) => {
                    let len = self.stream.read(buf)?;
                    self.auth.satisfy_read(len)?;
                }

                AuthWants::Write(bytes) => {
                    let len = self.stream.write(bytes)?;
                    if let Some(guid) = self.auth.satisfy_write(len)? {
                        return Ok(guid);
                    }
                }
            }
        }
    }

    pub(crate) fn send_message(&mut self, message: &mut Message) -> Result<()> {
        message.header.serial = self.serial.increment_and_get();
        self.writer.enqueue(&message)?;

        loop {
            let Some(buf) = self.writer.wants_write() else {
                break;
            };
            let len = self.stream.write(buf)?;
            self.writer.satisfy(len)?;
        }

        Ok(())
    }

    pub(crate) fn read_message(&mut self) -> Result<Message> {
        loop {
            let buf = self.reader.wants_read();
            let len = self.stream.read(buf)?;
            if let Some(message) = self.reader.satisfy(len)? {
                return Ok(message);
            }
        }
    }
}
