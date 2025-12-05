use crate::{
    Message,
    encoders::MessageEncoder,
    fsm::WriterFSM,
    io_uring_connection::{
        buffered_reader_fsm::BufferedReaderFSM,
        sqe::{READ_USER_DATA, WRITE_USER_DATA, read_sqe, write_sqe},
    },
    serial::Serial,
};
use anyhow::Result;
use io_uring::{cqueue::Entry as Cqe, squeue::Entry as Sqe};

#[derive(Debug)]
pub(crate) struct IoUringReaderWriterFSM {
    fd: i32,
    serial: Serial,
    reader: BufferedReaderFSM,
    writer: WriterFSM,
}

impl IoUringReaderWriterFSM {
    pub(crate) fn new(fd: i32, serial: Serial, queue: Vec<Vec<u8>>) -> Self {
        let mut writer = WriterFSM::new();
        for buf in queue {
            writer.enqueue(buf);
        }

        Self {
            fd,
            serial,
            reader: BufferedReaderFSM::new(),
            writer,
        }
    }

    pub(crate) fn enqueue(&mut self, message: &mut Message) -> Result<()> {
        *message.serial_mut() = self.serial.increment_and_get();
        let buf = MessageEncoder::encode(message)?;
        self.writer.enqueue(buf);
        Ok(())
    }

    pub(crate) fn next_sqe(&mut self) -> Option<Sqe> {
        if let Some(buf) = self.writer.wants() {
            return Some(write_sqe(self.fd, buf));
        }

        if let Some(buf) = self.reader.wants() {
            return Some(read_sqe(self.fd, buf));
        }

        None
    }

    pub(crate) fn process_cqe(&mut self, cqe: Cqe) -> Result<Option<Message>> {
        match cqe.user_data() {
            WRITE_USER_DATA => {
                let written = cqe.result();
                assert!(written >= 0);
                let written = written as usize;

                self.writer.satisfy(written)?;
                Ok(None)
            }

            READ_USER_DATA => {
                let read = cqe.result();
                assert!(read >= 0);
                let read = read as usize;

                if let Some(message) = self.reader.satisfy(read)? {
                    return Ok(Some(message));
                }
                Ok(None)
            }

            _ => Ok(None),
        }
    }
}
