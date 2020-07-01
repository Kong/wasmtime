use std::io::Result;
use std::os::unix::prelude::*;

use bitflags::bitflags;

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
        const OOB = libc::MSG_OOB;
        const PEEK = libc::MSG_PEEK;
        const WAIT_ALL = libc::MSG_WAITALL;
    }
}

bitflags! {
    pub struct SendFlags: libc::c_int {
        const OOB = libc::MSG_OOB;
        const DONTROUTE = libc::MSG_DONTROUTE;
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


pub unsafe fn accept(fd: RawFd) -> Result<(RawFd, SockAddr)> {
    let mut storage: libc::sockaddr_storage = std::mem::zeroed();
    let mut len = std::mem::size_of_val(&storage) as libc::socklen_t;

    let fd = from_result( libc::accept(fd, &mut storage as *mut _ as *mut _, &mut len) )?;

    Ok((fd, SockAddr { storage, len }))
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

pub unsafe fn send(fd: RawFd, buf: &[u8], flags: SendFlags) -> Result<usize> {
    let bufused = from_result(libc::send(
        fd,
        buf.as_ptr() as *const libc::c_void,
        buf.len(),
        flags.bits,
    ))?;
    Ok(bufused as usize)
}

pub unsafe fn sock_name(fd: RawFd) -> Result<SockAddr> {
    let mut storage: libc::sockaddr_storage = std::mem::zeroed();
    let mut len = std::mem::size_of_val(&storage) as libc::socklen_t;

    from_result(libc::getsockname(
        fd,
        &mut storage as *mut _ as *mut _,
        &mut len
    ))?;

    Ok(SockAddr { storage, len })
}

pub unsafe fn peer_name(fd: RawFd) -> Result<SockAddr> {
    let mut storage: libc::sockaddr_storage = std::mem::zeroed();
    let mut len = std::mem::size_of_val(&storage) as libc::socklen_t;

    from_result(libc::getpeername(
        fd,
        &mut storage as *mut _ as *mut _,
        &mut len
    ))?;

    Ok(SockAddr { storage, len })
}


pub unsafe fn shutdown(fd: RawFd, how: ShutdownMode) -> Result<()> {
    from_success_code(libc::shutdown(fd, how as libc::c_int))?;
    Ok(())
}

pub unsafe fn set_reuse_addr(fd: RawFd, reuse: bool) -> Result<()> {
    let reuse = &(reuse as libc::c_int) as *const libc::c_int as *const libc::c_void;
    let size = std::mem::size_of::<libc::c_int>() as libc::socklen_t;
    from_success_code( libc::setsockopt(fd, libc::SOL_SOCKET, libc::SO_REUSEADDR, reuse, size) )
}

pub unsafe fn get_reuse_addr(fd: RawFd) -> Result<bool> {
    let mut reuse: libc::c_int = std::mem::zeroed();
    let mut len = std::mem::size_of::<libc::c_int>() as libc::socklen_t;
    from_success_code( libc::getsockopt(fd, libc::SOL_SOCKET, libc::SO_REUSEADDR, &mut reuse as *mut _ as *mut _, &mut len) )?;
    Ok(reuse != 0)
}

pub unsafe fn set_reuse_port(fd: RawFd, reuse: bool) -> Result<()> {
    let reuse = &(reuse as libc::c_int) as *const libc::c_int as *const libc::c_void;
    let size = std::mem::size_of::<libc::c_int>() as libc::socklen_t;
    from_success_code( libc::setsockopt(fd, libc::SOL_SOCKET, libc::SO_REUSEPORT, reuse, size) )
}

pub unsafe fn get_reuse_port(fd: RawFd) -> Result<bool> {
    let mut reuse: libc::c_int = std::mem::zeroed();
    let mut len = std::mem::size_of::<libc::c_int>() as libc::socklen_t;
    from_success_code( libc::getsockopt(fd, libc::SOL_SOCKET, libc::SO_REUSEPORT, &mut reuse as *mut _ as *mut _, &mut len) )?;
    Ok(reuse != 0)
}

pub unsafe fn set_recv_buf_size(fd: RawFd, buf_size: u32) -> Result<()> {
    let buf_size = &(buf_size as libc::c_int) as *const libc::c_int as *const libc::c_void;
    let size = std::mem::size_of::<libc::c_int>() as libc::socklen_t;
    from_success_code( libc::setsockopt(fd, libc::SOL_SOCKET, libc::SO_RCVBUF, buf_size, size) )
}

pub unsafe fn get_recv_buf_size(fd: RawFd) -> Result<u32> {
    let mut buf_size: libc::c_int = std::mem::zeroed();
    let mut len = std::mem::size_of::<libc::c_int>() as libc::socklen_t;
    from_success_code( libc::getsockopt(fd, libc::SOL_SOCKET, libc::SO_RCVBUF, &mut buf_size as *mut _ as *mut _, &mut len) )?;
    Ok(buf_size as u32)
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
