use anyhow::{Context, Result};
use libc::{AF_UNIX, sockaddr_un};

#[derive(Debug)]
pub(crate) enum ConnectFSM {
    None,
    Socket { fd: i32, addr: sockaddr_un },
}

pub(crate) enum ConnectWants {
    Socket,
    Connect { fd: i32, addr: *const sockaddr_un },
}

impl ConnectFSM {
    pub(crate) fn new() -> Self {
        Self::None
    }

    pub(crate) fn wants(&mut self) -> ConnectWants {
        match self {
            Self::None => ConnectWants::Socket,
            Self::Socket { fd, addr } => ConnectWants::Connect { fd: *fd, addr },
        }
    }

    pub(crate) fn satisfy_socket(&mut self, fd: i32) -> Result<()> {
        let Self::None = self else {
            panic!("malformed state, {self:?}")
        };

        *self = Self::Socket {
            fd,
            addr: addr_to_connect()?,
        };
        Ok(())
    }

    pub(crate) fn satisfy_connect(&mut self) -> i32 {
        let Self::Socket { fd, .. } = self else {
            panic!("malformed state, {self:?}")
        };
        let fd = *fd;
        *self = Self::None;
        fd
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
