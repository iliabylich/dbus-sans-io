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

enum IoUringFSM {
    Connect(IoUringConnectFSM),
    Auth(IoUringAuthFSM),
    ReaderWriter(IoUringReaderWriterFSM),
}

pub struct IoUringConnection {
    fsm: IoUringFSM,
}

impl IoUringConnection {
    pub fn session() -> Self {
        Self {
            fsm: IoUringFSM::Connect(IoUringConnectFSM::new()),
        }
    }

    pub fn enqueue(&mut self, message: &mut Message) -> Result<()> {
        match &mut self.fsm {
            IoUringFSM::Connect(connector) => connector.enqueue(message),
            IoUringFSM::Auth(auth) => auth.enqueue(message),
            IoUringFSM::ReaderWriter(rw) => rw.enqueue(message),
        }
    }

    pub fn next_sqe(&mut self) -> Option<Sqe> {
        match &mut self.fsm {
            IoUringFSM::Connect(connector) => Some(connector.next_sqe()),
            IoUringFSM::Auth(auth) => Some(auth.next_sqe()),
            IoUringFSM::ReaderWriter(rw) => rw.next_sqe(),
        }
    }

    pub fn process_cqe(&mut self, cqe: Cqe) -> Result<Option<Message>> {
        macro_rules! take_fsm {
            () => {{
                let mut zeroed =
                    unsafe { std::mem::MaybeUninit::<IoUringFSM>::zeroed().assume_init() };
                std::mem::swap(&mut self.fsm, &mut zeroed);
                zeroed
            }};
        }

        match &mut self.fsm {
            IoUringFSM::Connect(connector) => match connector.process_cqe(cqe)? {
                Some(fd) => {
                    let IoUringFSM::Connect(IoUringConnectFSM { serial, queue, .. }) = take_fsm!()
                    else {
                        unreachable!()
                    };

                    self.fsm = IoUringFSM::Auth(IoUringAuthFSM::new(fd, serial, queue));
                    Ok(None)
                }
                None => Ok(None),
            },

            IoUringFSM::Auth(auth) => match auth.process_cqe(cqe)? {
                Some(_guid) => {
                    let IoUringFSM::Auth(IoUringAuthFSM {
                        fd, serial, queue, ..
                    }) = take_fsm!()
                    else {
                        unreachable!()
                    };
                    self.fsm =
                        IoUringFSM::ReaderWriter(IoUringReaderWriterFSM::new(fd, serial, queue));
                    Ok(None)
                }
                None => Ok(None),
            },

            IoUringFSM::ReaderWriter(rw) => rw.process_cqe(cqe),
        }
    }
}
