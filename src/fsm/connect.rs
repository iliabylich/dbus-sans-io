use libc::{AF_UNIX, sockaddr_un};

#[derive(Debug)]
pub(crate) enum ConnectFSM {
    None,
    Socket { fd: i32, addr: sockaddr_un },
    Connected(i32),
}

pub(crate) enum ConnectWants {
    Socket,
    Connect { fd: i32, addr: *const sockaddr_un },
}

impl ConnectFSM {
    pub(crate) fn new() -> Self {
        Self::None
    }

    pub(crate) fn wants(&mut self) -> Option<ConnectWants> {
        match self {
            Self::None => Some(ConnectWants::Socket),
            Self::Socket { fd, addr } => Some(ConnectWants::Connect { fd: *fd, addr }),
            Self::Connected(_) => None,
        }
    }

    pub(crate) fn satisfy_socket(&mut self, fd: i32) {
        let Self::None = self else {
            panic!("malformed state, {self:?}")
        };

        *self = Self::Socket {
            fd,
            addr: addr_to_connect(),
        }
    }

    pub(crate) fn satisfy_connect(&mut self) {
        let Self::Socket { fd, .. } = self else {
            panic!("malformed state, {self:?}")
        };
        let fd = *fd;
        *self = Self::Connected(fd);
    }

    pub(crate) fn fd(&self) -> Option<i32> {
        if let Self::Connected(fd) = self {
            Some(*fd)
        } else {
            None
        }
    }
}

fn addr_to_connect() -> sockaddr_un {
    let address = std::env::var("DBUS_SESSION_BUS_ADDRESS").expect("no DBUS_SESSION_BUS_ADDRESS");
    let (_, path) = address.split_once("=").expect("no = separator");
    let path = {
        let mut out = [0; 108];
        out[..path.len()].copy_from_slice(unsafe { std::mem::transmute(path.as_bytes()) });
        out
    };
    sockaddr_un {
        sun_family: AF_UNIX as u16,
        sun_path: path,
    }
}
