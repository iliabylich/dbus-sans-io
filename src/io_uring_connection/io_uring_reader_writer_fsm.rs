use crate::{
    Message,
    encoders::MessageEncoder,
    fsm::WriterFSM,
    io_uring_connection::{
        buffered_reader_fsm::BufferedReaderFSM,
        sqe::{read_sqe, write_sqe},
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
    read_user_data: u64,
    write_user_data: u64,
}

impl IoUringReaderWriterFSM {
    pub(crate) fn new(
        fd: i32,
        serial: Serial,
        queue: Vec<Vec<u8>>,
        read_user_data: u64,
        write_user_data: u64,
    ) -> Self {
        let mut writer = WriterFSM::new();
        for buf in queue {
            writer.enqueue(buf);
        }

        Self {
            fd,
            serial,
            reader: BufferedReaderFSM::new(),
            writer,
            read_user_data,
            write_user_data,
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
            return Some(write_sqe(self.fd, buf, self.write_user_data));
        }

        if let Some(buf) = self.reader.wants() {
            return Some(read_sqe(self.fd, buf, self.read_user_data));
        }

        None
    }

    pub(crate) fn process_cqe(&mut self, cqe: Cqe) -> Result<Option<Message>> {
        match cqe.user_data() {
            data if data == self.write_user_data => {
                let written = cqe.result();
                assert!(written >= 0);
                let written = written as usize;

                self.writer.satisfy(written)?;
                Ok(None)
            }

            data if data == self.read_user_data => {
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
