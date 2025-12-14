use io_uring::{opcode, squeue::Entry as Sqe, types};
use libc::{AF_UNIX, SOCK_STREAM, sockaddr, sockaddr_un};

pub(crate) fn socket_sqe(user_data: u64) -> Sqe {
    opcode::Socket::new(AF_UNIX, SOCK_STREAM, 0)
        .build()
        .user_data(user_data)
}

pub(crate) fn connect_sqe(fd: i32, addr: *const sockaddr_un, user_data: u64) -> Sqe {
    opcode::Connect::new(
        types::Fd(fd),
        addr.cast::<sockaddr>(),
        std::mem::size_of::<sockaddr_un>() as u32,
    )
    .build()
    .user_data(user_data)
}

pub(crate) fn write_sqe(fd: i32, buf: &[u8], user_data: u64) -> Sqe {
    opcode::Write::new(types::Fd(fd), buf.as_ptr(), buf.len() as u32)
        .build()
        .user_data(user_data)
}

pub(crate) fn read_sqe(fd: i32, buf: &mut [u8], user_data: u64) -> Sqe {
    opcode::Read::new(types::Fd(fd), buf.as_mut_ptr(), buf.len() as u32)
        .build()
        .user_data(user_data)
}
