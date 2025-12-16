use libc::{AF_UNIX, SOCK_STREAM, sockaddr, sockaddr_un};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Sqe {
    Socket {
        domain: i32,
        socket_type: i32,
        protocol: i32,
        user_data: u64,
    },

    Connect {
        fd: i32,
        addr: *const sockaddr,
        addrlen: u32,
        user_data: u64,
    },

    Write {
        fd: i32,
        buf: *const u8,
        len: u32,
        user_data: u64,
    },

    Read {
        fd: i32,
        buf: *mut u8,
        len: u32,
        user_data: u64,
    },
}

impl Sqe {
    pub(crate) fn user_data(self) -> u64 {
        match self {
            Self::Socket { user_data, .. }
            | Self::Connect { user_data, .. }
            | Self::Write { user_data, .. }
            | Self::Read { user_data, .. } => user_data,
        }
    }
}

pub(crate) fn socket_sqe(user_data: u64) -> Sqe {
    Sqe::Socket {
        domain: AF_UNIX,
        socket_type: SOCK_STREAM,
        protocol: 0,
        user_data,
    }
}

pub(crate) fn connect_sqe(fd: i32, addr: *const sockaddr_un, user_data: u64) -> Sqe {
    Sqe::Connect {
        fd,
        addr: addr.cast::<sockaddr>(),
        addrlen: std::mem::size_of::<sockaddr_un>() as u32,
        user_data,
    }
}

pub(crate) fn write_sqe(fd: i32, buf: &[u8], user_data: u64) -> Sqe {
    Sqe::Write {
        fd,
        buf: buf.as_ptr(),
        len: buf.len() as u32,
        user_data,
    }
}

pub(crate) fn read_sqe(fd: i32, buf: &mut [u8], user_data: u64) -> Sqe {
    Sqe::Read {
        fd,
        buf: buf.as_mut_ptr(),
        len: buf.len() as u32,
        user_data,
    }
}
