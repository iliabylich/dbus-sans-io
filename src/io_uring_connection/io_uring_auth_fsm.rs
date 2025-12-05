use crate::{
    Message,
    encoders::MessageEncoder,
    fsm::{AuthFSM, AuthWants},
    io_uring_connection::sqe::{READ_USER_DATA, WRITE_USER_DATA, read_sqe, write_sqe},
    serial::Serial,
};
use anyhow::Result;
use io_uring::{cqueue::Entry as Cqe, squeue::Entry as Sqe};

pub(crate) struct IoUringAuthFSM {
    pub(crate) fd: i32,
    pub(crate) serial: Serial,
    pub(crate) queue: Vec<Vec<u8>>,
    pub(crate) auth: AuthFSM,
}

impl IoUringAuthFSM {
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

    pub(crate) fn process_cqe(&mut self, cqe: Cqe) -> Result<Option<()>> {
        match cqe.user_data() {
            WRITE_USER_DATA => {
                let written = cqe.result();
                assert!(written >= 0);
                let written = written as usize;

                if let Some(_guid) = self.auth.satisfy_write(written)? {
                    return Ok(Some(()));
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
