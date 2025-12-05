use crate::{
    Message,
    encoders::MessageEncoder,
    io_uring_connection::sqe::{CONNECT_USER_DATA, SOCKET_USER_DATA, connect_sqe, socket_sqe},
    serial::Serial,
};
use anyhow::{Context as _, Result};
use io_uring::{cqueue::Entry as Cqe, squeue::Entry as Sqe};
use libc::{AF_UNIX, sockaddr_un};

#[derive(Debug)]
pub(crate) struct IoUringConnectFSM {
    fd_and_socket: Option<(i32, sockaddr_un)>,
    pub(crate) serial: Serial,
    pub(crate) queue: Vec<Vec<u8>>,
}

impl IoUringConnectFSM {
    pub(crate) fn new() -> Self {
        Self {
            fd_and_socket: None,
            serial: Serial::zero(),
            queue: vec![],
        }
    }

    pub(crate) fn enqueue(&mut self, message: &mut Message) -> Result<()> {
        *message.serial_mut() = self.serial.increment_and_get();
        let buf = MessageEncoder::encode(message)?;
        self.queue.push(buf);
        Ok(())
    }

    pub(crate) fn next_sqe(&mut self) -> Sqe {
        match self.fd_and_socket.as_ref() {
            None => socket_sqe(),
            Some((fd, addr)) => connect_sqe(*fd, addr),
        }
    }

    pub(crate) fn process_cqe(&mut self, cqe: Cqe) -> Result<Option<i32>> {
        match cqe.user_data() {
            SOCKET_USER_DATA => {
                let fd = cqe.result();
                assert!(fd > 0);

                let None = self.fd_and_socket.take() else {
                    panic!("malformed state, {self:?}")
                };

                self.fd_and_socket = Some((fd, addr_to_connect()?));

                Ok(None)
            }

            CONNECT_USER_DATA => {
                assert!(cqe.result() >= 0);

                let Some((fd, _)) = self.fd_and_socket.take() else {
                    panic!("malformed state, {self:?}")
                };
                self.fd_and_socket = None;
                Ok(Some(fd))
            }

            _ => Ok(None),
        }
    }
}

fn addr_to_connect() -> Result<sockaddr_un> {
    let address =
        std::env::var("DBUS_SESSION_BUS_ADDRESS").context("no DBUS_SESSION_BUS_ADDRESS")?;
    let (_, path) = address.split_once("=").context("no = separator")?;
    let path = {
        let mut out = [0; 108];
        out[..path.len()]
            .copy_from_slice(unsafe { std::mem::transmute::<&[u8], &[i8]>(path.as_bytes()) });
        out
    };
    Ok(sockaddr_un {
        sun_family: AF_UNIX as u16,
        sun_path: path,
    })
}
