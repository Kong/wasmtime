use crate::{from_success_code, from_result};
use std::io::Result;
use std::os::unix::prelude::*;

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AddressFamily {
    InetAddr = libc::PF_INET,
    Inet6Addr = libc::PF_INET6
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SockProtocol {
    Tcp = libc::IPPROTO_TCP,
    Udp = libc::IPPROTO_UDP
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

pub unsafe fn socket(af: AddressFamily, t: SockType, p: Option<SockProtocol>) -> Result<RawFd> {
    let protocol: libc::c_int = match p {
        None => 0,
        Some(p) => p as libc::c_int,
    };
    from_result(libc::socket(af as libc::c_int, t as libc::c_int, protocol))
}

pub unsafe fn get_socket_type(fd: RawFd) -> Result<SockType> {
    use std::mem::{self, MaybeUninit};
    let mut buffer = MaybeUninit::<SockType>::zeroed().assume_init();
    let mut out_len = mem::size_of::<SockType>() as libc::socklen_t;
    from_success_code(libc::getsockopt(
        fd,
        libc::SOL_SOCKET,
        libc::SO_TYPE,
        &mut buffer as *mut SockType as *mut _,
        &mut out_len,
    ))?;
    assert_eq!(
        out_len as usize,
        mem::size_of::<SockType>(),
        "invalid SockType value"
    );
    Ok(buffer)
}
