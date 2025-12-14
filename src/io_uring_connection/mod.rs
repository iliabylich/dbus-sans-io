use crate::Message;
use anyhow::Result;
use io_uring::{cqueue::Entry as Cqe, squeue::Entry as Sqe};
use io_uring_auth_fsm::IoUringAuthFSM;
use io_uring_connect_fsm::IoUringConnectFSM;
use io_uring_reader_writer_fsm::IoUringReaderWriterFSM;

mod buffered_reader_fsm;
mod io_uring_auth_fsm;
mod io_uring_connect_fsm;
mod io_uring_reader_writer_fsm;
mod sqe;

#[derive(Default)]
enum IoUringFSM {
    Connect(IoUringConnectFSM),
    Auth(IoUringAuthFSM),
    ReaderWriter(IoUringReaderWriterFSM),
    #[default]
    None,
}

pub struct IoUringConnection {
    read_user_data: u64,
    write_user_data: u64,

    fsm: IoUringFSM,
}

impl IoUringConnection {
    pub fn session(
        socket_user_data: u64,
        connect_user_data: u64,
        read_user_data: u64,
        write_user_data: u64,
    ) -> Self {
        Self {
            read_user_data,
            write_user_data,

            fsm: IoUringFSM::Connect(IoUringConnectFSM::new(socket_user_data, connect_user_data)),
        }
    }

    pub fn enqueue(&mut self, message: &mut Message) -> Result<()> {
        match &mut self.fsm {
            IoUringFSM::Connect(connector) => connector.enqueue(message),
            IoUringFSM::Auth(auth) => auth.enqueue(message),
            IoUringFSM::ReaderWriter(rw) => rw.enqueue(message),
            IoUringFSM::None => unreachable!(),
        }
    }

    pub fn next_sqe(&mut self) -> Option<Sqe> {
        match &mut self.fsm {
            IoUringFSM::Connect(connector) => Some(connector.next_sqe()),
            IoUringFSM::Auth(auth) => Some(auth.next_sqe()),
            IoUringFSM::ReaderWriter(rw) => rw.next_sqe(),
            IoUringFSM::None => unreachable!(),
        }
    }

    fn take_fsm(&mut self) -> IoUringFSM {
        std::mem::take(&mut self.fsm)
    }

    pub fn process_cqe(&mut self, cqe: Cqe) -> Result<Option<Message>> {
        match &mut self.fsm {
            IoUringFSM::Connect(connector) => match connector.process_cqe(cqe)? {
                Some(fd) => {
                    let IoUringFSM::Connect(IoUringConnectFSM { serial, queue, .. }) =
                        self.take_fsm()
                    else {
                        unreachable!()
                    };

                    self.fsm = IoUringFSM::Auth(IoUringAuthFSM::new(
                        fd,
                        serial,
                        queue,
                        self.read_user_data,
                        self.write_user_data,
                    ));
                    Ok(None)
                }
                None => Ok(None),
            },

            IoUringFSM::Auth(auth) => match auth.process_cqe(cqe)? {
                Some(_guid) => {
                    let IoUringFSM::Auth(IoUringAuthFSM {
                        fd, serial, queue, ..
                    }) = self.take_fsm()
                    else {
                        unreachable!()
                    };
                    self.fsm = IoUringFSM::ReaderWriter(IoUringReaderWriterFSM::new(
                        fd,
                        serial,
                        queue,
                        self.read_user_data,
                        self.write_user_data,
                    ));
                    Ok(None)
                }
                None => Ok(None),
            },

            IoUringFSM::ReaderWriter(rw) => rw.process_cqe(cqe),

            IoUringFSM::None => unreachable!(),
        }
    }
}
