use crate::{
    encoders::MessageEncoder,
    fsm::{AuthFSM, AuthWants, ReaderFSM, WriterFSM},
    serial::Serial,
    session_connection,
    types::Message,
};
use anyhow::Result;
use std::{
    io::{Read as _, Write as _},
    os::{fd::FromRawFd, unix::net::UnixStream},
};

pub struct BlockingConnection {
    stream: UnixStream,
    serial: Serial,

    auth: AuthFSM,
    reader: ReaderFSM,
    writer: WriterFSM,
}

impl BlockingConnection {
    pub fn session() -> Result<Self> {
        Ok(Self {
            stream: session_connection()?,
            serial: Serial::zero(),

            auth: AuthFSM::new(),
            reader: ReaderFSM::new(),
            writer: WriterFSM::new(),
        })
    }

    pub fn from_fd(fd: i32) -> Self {
        Self {
            stream: unsafe { UnixStream::from_raw_fd(fd) },

            serial: Serial::zero(),
            auth: AuthFSM::new(),
            reader: ReaderFSM::new(),
            writer: WriterFSM::new(),
        }
    }

    pub fn auth(&mut self) -> Result<()> {
        loop {
            match self.auth.wants() {
                AuthWants::Read(buf) => {
                    let len = self.stream.read(buf)?;
                    self.auth.satisfy_read(len)?;
                }

                AuthWants::Write(bytes) => {
                    let len = self.stream.write(bytes)?;
                    if let Some(_guid) = self.auth.satisfy_write(len)? {
                        return Ok(());
                    }
                }
            }
        }
    }

    pub fn send_message(&mut self, message: &mut Message) -> Result<()> {
        *message.serial_mut() = self.serial.increment_and_get();

        let buf = MessageEncoder::encode(message)?;
        self.writer.enqueue(buf);

        loop {
            let Some(buf) = self.writer.wants() else {
                break;
            };
            let len = self.stream.write(buf)?;
            self.writer.satisfy(len)?;
        }

        Ok(())
    }

    pub fn read_message(&mut self) -> Result<Message> {
        loop {
            let buf = self.reader.wants();
            let len = self.stream.read(buf)?;
            if let Some(message) = self.reader.satisfy(len)? {
                return Ok(message);
            }
        }
    }
}
