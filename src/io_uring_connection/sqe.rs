use io_uring::{opcode, squeue::Entry as Sqe, types};
use libc::{AF_UNIX, SOCK_STREAM, sockaddr, sockaddr_un};

pub(crate) const SOCKET_USER_DATA: u64 = 1;
pub(crate) const CONNECT_USER_DATA: u64 = 2;
pub(crate) const READ_USER_DATA: u64 = 3;
pub(crate) const WRITE_USER_DATA: u64 = 4;

pub(crate) fn socket_sqe() -> Sqe {
    opcode::Socket::new(AF_UNIX, SOCK_STREAM, 0)
        .build()
        .user_data(SOCKET_USER_DATA)
}

pub(crate) fn connect_sqe(fd: i32, addr: *const sockaddr_un) -> Sqe {
    opcode::Connect::new(
        types::Fd(fd),
        addr.cast::<sockaddr>(),
        std::mem::size_of::<sockaddr_un>() as u32,
    )
    .build()
    .user_data(CONNECT_USER_DATA)
}

pub(crate) fn write_sqe(fd: i32, buf: &[u8]) -> Sqe {
    opcode::Write::new(types::Fd(fd), buf.as_ptr(), buf.len() as u32)
        .build()
        .user_data(WRITE_USER_DATA)
}

pub(crate) fn read_sqe(fd: i32, buf: &mut [u8]) -> Sqe {
    opcode::Read::new(types::Fd(fd), buf.as_mut_ptr(), buf.len() as u32)
        .build()
        .user_data(READ_USER_DATA)
}
