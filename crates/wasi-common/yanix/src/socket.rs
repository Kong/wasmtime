use std::io::Result;
use std::os::unix::prelude::*;

use crate::{from_result, from_success_code};

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AddressFamily {
    InetAddr = libc::PF_INET,
    Inet6Addr = libc::PF_INET6,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SockProtocol {
    Tcp = libc::IPPROTO_TCP,
    Udp = libc::IPPROTO_UDP,
}

#[derive(Debug, Clone, Copy)]
#[repr(i32)]
pub enum SockType {
    Stream = libc::SOCK_STREAM,
    Datagram = libc::SOCK_DGRAM,
    SeqPacket = libc::SOCK_SEQPACKET,
    Raw = libc::SOCK_RAW,
    Rdm = libc::SOCK_RDM,
}

#[derive(Debug, Clone, Copy)]
pub struct InAddr(pub libc::in_addr);

#[derive(Debug, Clone, Copy)]
pub struct In6Addr(pub libc::in6_addr);

#[derive(Debug, Clone, Copy)]
pub struct SockAddr {
    pub storage: libc::sockaddr_storage,
    pub len: libc::socklen_t,
}

impl SockAddr {
    fn as_ptr(&self) -> *const libc::sockaddr {
        &self.storage as *const _ as *const _
    }

    fn len(&self) -> libc::socklen_t {
        self.len
    }
}

pub unsafe fn socket(af: AddressFamily, t: SockType, p: Option<SockProtocol>) -> Result<RawFd> {
    let protocol: libc::c_int = match p {
        None => 0,
        Some(p) => p as libc::c_int,
    };
    from_result(libc::socket(af as libc::c_int, t as libc::c_int, protocol))
}

pub unsafe fn listen(fd: RawFd, backlog: usize) -> Result<()> {
    from_success_code(libc::listen(fd, backlog as libc::c_int))
}

pub unsafe fn bind(fd: RawFd, addr: &SockAddr) -> Result<()> {
    from_success_code(libc::bind(fd, addr.as_ptr(), addr.len()) )
}

pub unsafe fn connect(fd: RawFd, addr: &SockAddr) -> Result<()> {
    from_success_code(libc::connect(fd, addr.as_ptr(), addr.len()) )
}

pub unsafe fn get_socket_type(fd: RawFd) -> Result<SockType> {
    use std::mem::MaybeUninit;
    let mut buffer = MaybeUninit::<SockType>::zeroed().assume_init();
    let mut out_len = std::mem::size_of::<SockType>() as libc::socklen_t;
    from_success_code(libc::getsockopt(
        fd,
        libc::SOL_SOCKET,
        libc::SO_TYPE,
        &mut buffer as *mut SockType as *mut _,
        &mut out_len,
    ))?;
    assert_eq!(
        out_len as usize,
        std::mem::size_of::<SockType>(),
        "invalid SockType value"
    );
    Ok(buffer)
}
