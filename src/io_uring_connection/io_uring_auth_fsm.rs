use crate::{
    Cqe, Message, Sqe,
    encoders::MessageEncoder,
    fsm::{AuthFSM, AuthWants},
    io_uring_connection::sqe::{read_sqe, write_sqe},
    serial::Serial,
};
use anyhow::Result;

pub(crate) struct IoUringAuthFSM {
    pub(crate) fd: i32,
    pub(crate) serial: Serial,
    pub(crate) queue: Vec<Vec<u8>>,
    pub(crate) auth: AuthFSM,
    read_user_data: u64,
    write_user_data: u64,
}

impl IoUringAuthFSM {
    pub(crate) fn new(
        fd: i32,
        serial: Serial,
        queue: Vec<Vec<u8>>,
        read_user_data: u64,
        write_user_data: u64,
    ) -> Self {
        Self {
            fd,
            serial,
            queue,
            auth: AuthFSM::new(),
            read_user_data,
            write_user_data,
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
            AuthWants::Read(buf) => read_sqe(self.fd, buf, self.read_user_data),
            AuthWants::Write(buf) => write_sqe(self.fd, buf, self.write_user_data),
        }
    }

    pub(crate) fn process_cqe(&mut self, cqe: Cqe) -> Result<Option<()>> {
        match cqe.user_data {
            data if data == self.write_user_data => {
                assert!(cqe.result >= 0);
                let written = cqe.result as usize;

                if let Some(_guid) = self.auth.satisfy_write(written)? {
                    return Ok(Some(()));
                }
                Ok(None)
            }

            data if data == self.read_user_data => {
                assert!(cqe.result >= 0);
                let read = cqe.result as usize;

                self.auth.satisfy_read(read)?;
                Ok(None)
            }

            _ => Ok(None),
        }
    }
}
