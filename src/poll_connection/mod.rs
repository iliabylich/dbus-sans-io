use crate::{encoders::MessageEncoder, serial::Serial, session_connection, types::Message};
use anyhow::Result;
use std::os::fd::AsRawFd;

mod non_blocking_stream;
use non_blocking_stream::NonBlockingUnixStream;

mod poll_auth_fsm;
use poll_auth_fsm::PollAuthFSM;

mod poll_reader_writer_fsm;
use poll_reader_writer_fsm::PollReaderWriterFSM;

enum PollFSM {
    Auth(PollAuthFSM),
    ReaderWriter(PollReaderWriterFSM),
}

pub struct PollConnection {
    serial: Serial,
    fsm: PollFSM,
}

impl AsRawFd for PollConnection {
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        match &self.fsm {
            PollFSM::Auth(auth) => auth.as_raw_fd(),
            PollFSM::ReaderWriter(rw) => rw.as_raw_fd(),
        }
    }
}

impl PollConnection {
    pub fn session() -> Result<Self> {
        let stream = session_connection()?;
        stream.set_nonblocking(true)?;

        Ok(Self {
            serial: Serial::zero(),
            fsm: PollFSM::Auth(PollAuthFSM::new(NonBlockingUnixStream::new(stream))),
        })
    }

    pub fn enqueue(&mut self, message: &mut Message) -> Result<()> {
        *message.serial_mut() = self.serial.increment_and_get();
        let buf = MessageEncoder::encode(message)?;

        match &mut self.fsm {
            PollFSM::Auth(auth) => auth.enqueue(buf),
            PollFSM::ReaderWriter(rw) => rw.enqueue(buf),
        }

        Ok(())
    }

    pub fn events(&self) -> i16 {
        match &self.fsm {
            PollFSM::Auth(auth) => auth.events(),
            PollFSM::ReaderWriter(rw) => rw.events(),
        }
    }

    pub fn poll(&mut self, readable: bool, writable: bool) -> Result<Vec<Message>> {
        macro_rules! take_fsm {
            () => {{
                let mut zeroed =
                    unsafe { std::mem::MaybeUninit::<PollFSM>::zeroed().assume_init() };
                std::mem::swap(&mut self.fsm, &mut zeroed);
                zeroed
            }};
        }

        match &mut self.fsm {
            PollFSM::Auth(auth) => {
                if auth.poll(readable, writable)? {
                    // EOA
                    let PollFSM::Auth(PollAuthFSM { stream, queue, .. }) = take_fsm!() else {
                        unreachable!()
                    };

                    self.fsm = PollFSM::ReaderWriter(PollReaderWriterFSM::new(stream, queue));
                }

                Ok(vec![])
            }
            PollFSM::ReaderWriter(rw) => rw.poll(readable, writable),
        }
    }
}
