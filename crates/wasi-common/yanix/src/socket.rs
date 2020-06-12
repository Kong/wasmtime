use std::io::Result;
use bitflags::bitflags;
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

bitflags! {
    pub struct RecvFlags: libc::c_int {
        const PEEK = libc::MSG_PEEK;
        const WAIT_ALL = libc::MSG_WAITALL;
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(i32)]
pub enum ShutdownMode {
    Write = libc::SHUT_WR,
    Read = libc::SHUT_RD,
    Both = libc::SHUT_RDWR,
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
    from_success_code(libc::bind(fd, addr.as_ptr(), addr.len()))
}

pub unsafe fn connect(fd: RawFd, addr: &SockAddr) -> Result<()> {
    from_success_code(libc::connect(fd, addr.as_ptr(), addr.len()))
}

pub unsafe fn recv(fd: RawFd, buf: &mut [u8], flags: RecvFlags) -> Result<usize> {
    let bufused = from_result(libc::recv(
        fd,
        buf.as_mut_ptr() as *mut libc::c_void,
        buf.len(),
        flags.bits,
    ))?;
    Ok(bufused as usize)
}

pub unsafe fn shutdown(fd: RawFd, how: ShutdownMode) -> Result<()> {
    from_success_code(libc::shutdown(fd, how as libc::c_int) )?;
    Ok(())
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
