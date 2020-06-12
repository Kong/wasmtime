use std::io;
use std::convert::TryFrom;
use yanix::socket::socket;

use crate::wasi::types;
use std::os::unix::prelude::{AsRawFd, FromRawFd, IntoRawFd, RawFd};

#[derive(Debug)]
pub(crate) struct RawOsSocket(RawFd);

impl AsRawFd for RawOsSocket {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

impl IntoRawFd for RawOsSocket {
    fn into_raw_fd(self) -> RawFd {
        self.0
    }
}

impl FromRawFd for RawOsSocket {
    unsafe fn from_raw_fd(raw: RawFd) -> Self {
        Self(raw)
    }
}

impl From<types::AddressFamily> for yanix::socket::AddressFamily {
    fn from(af: types::AddressFamily) -> Self {
        match af {
            types::AddressFamily::Inet4 => yanix::socket::AddressFamily::InetAddr,
            types::AddressFamily::Inet6 => yanix::socket::AddressFamily::Inet6Addr
        }
    }
}

impl From<types::SockType> for yanix::socket::SockType {
    fn from(t: types::SockType) -> Self {
        match t {
            types::SockType::SocketStream => yanix::socket::SockType::Stream,
            types::SockType::SocketDgram => yanix::socket::SockType::Datagram
        }
    }
}

impl From<&types::AddrIp4> for yanix::socket::InAddr {
    fn from(addr: &types::AddrIp4) -> Self {
        let ip = (((addr.n0 as u32) << 24) |
            ((addr.n1 as u32) << 16) |
            ((addr.h0 as u32) << 8) |
            ((addr.h1 as u32) << 0)).to_be();

        yanix::socket::InAddr(libc::in_addr { s_addr: ip })
    }
}

impl From<&types::AddrIp6> for yanix::socket::In6Addr {
    fn from(addr: &types::AddrIp6) -> Self {
        yanix::socket::In6Addr(libc::in6_addr {
            s6_addr: [(addr.n0 >> 8) as u8, (addr.n0 & 0xff) as u8,
                (addr.n1 >> 8) as u8, (addr.n1 & 0xff) as u8,
                (addr.n2 >> 8) as u8, (addr.n2 & 0xff) as u8,
                (addr.n3 >> 8) as u8, (addr.n3 & 0xff) as u8,
                (addr.h0 >> 8) as u8, (addr.h0 & 0xff) as u8,
                (addr.h1 >> 8) as u8, (addr.h1 & 0xff) as u8,
                (addr.h2 >> 8) as u8, (addr.h2 & 0xff) as u8,
                (addr.h3 >> 8) as u8, (addr.h3 & 0xff) as u8]
        })
    }
}

impl From<&types::Addr> for yanix::socket::SockAddr {
    fn from(t: &types::Addr) -> Self {
        match t {
            types::Addr::Ip4(addr) => {
                use std::mem::MaybeUninit;
                let mut storage = MaybeUninit::<libc::sockaddr_storage>::zeroed();
                let ptr = storage.as_mut_ptr() as *mut libc::sockaddr_in;
                unsafe {
                    (*ptr).sin_family = yanix::socket::AddressFamily::InetAddr as libc::sa_family_t;
                    (*ptr).sin_port = addr.port.to_be();
                    (*ptr).sin_addr = yanix::socket::InAddr::from(&addr.addr).0;
                };
                yanix::socket::SockAddr {
                    storage: unsafe { storage.assume_init() },
                    len: std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t
                }
            },
            types::Addr::Ip6(addr) => {
                use std::mem::MaybeUninit;
                let mut storage = MaybeUninit::<libc::sockaddr_storage>::zeroed();
                let ptr = storage.as_mut_ptr() as *mut libc::sockaddr_in6;
                unsafe {
                    (*ptr).sin6_family = yanix::socket::AddressFamily::Inet6Addr as libc::sa_family_t;
                    (*ptr).sin6_port = addr.port.to_be();
                    (*ptr).sin6_addr = yanix::socket::In6Addr::from(&addr.addr).0;
                };
                yanix::socket::SockAddr {
                    storage: unsafe { storage.assume_init() },
                    len: std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t
                }
            }
        }
    }
}

impl From<types::Riflags> for yanix::socket::RecvFlags {
    fn from(flags: types::Riflags) -> Self {
        let mut out = yanix::socket::RecvFlags::empty();
        if flags & types::Riflags::RECV_PEEK == types::Riflags::RECV_PEEK {
            out |= yanix::socket::RecvFlags::PEEK;
        }
        if flags & types::Riflags::RECV_WAITALL == types::Riflags::RECV_WAITALL {
            out |= yanix::socket::RecvFlags::WAIT_ALL;
        }
        out
    }
}

impl TryFrom<types::Sdflags> for yanix::socket::ShutdownMode {
    type Error = std::io::Error;

    fn try_from(flags: types::Sdflags) -> io::Result<Self> {
        if (flags & types::Sdflags::RD == types::Sdflags::RD) && (flags & types::Sdflags::WR == types::Sdflags::WR) {
            Ok(yanix::socket::ShutdownMode::Both)
        } else if flags & types::Sdflags::RD == types::Sdflags::RD {
            Ok(yanix::socket::ShutdownMode::Read)
        } else if flags & types::Sdflags::WR == types::Sdflags::WR {
            Ok(yanix::socket::ShutdownMode::Write)
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "bad flag"))
        }
    }
}

impl RawOsSocket {
    /// Tries clone `self`.
    pub(crate) fn try_clone(&self) -> io::Result<Self> {
        Ok(unsafe { RawOsSocket::from_raw_fd(self.0) })
    }

    pub(crate) fn new(address_family: types::AddressFamily, socket_type: types::SockType) -> io::Result<Self> {
        let address_family = yanix::socket::AddressFamily::from(address_family);
        let c_socket_type = yanix::socket::SockType::from(socket_type);
        let fd = unsafe { socket(address_family, c_socket_type, None)? };
        Ok(unsafe { RawOsSocket::from_raw_fd(fd) })
    }

    pub(crate) fn connect(&self, addr: &types::Addr) -> io::Result<()> {
        let addr = yanix::socket::SockAddr::from(addr);
        unsafe { yanix::socket::connect(self.as_raw_fd(), &addr ) }
    }

    pub(crate) fn recv(&self, buf: &mut [u8], flags: types::Riflags) -> io::Result<usize> {
        unsafe { yanix::socket::recv(self.as_raw_fd(), buf, yanix::socket::RecvFlags::from(flags) ) }
    }

    pub(crate) fn shutdown(&self, how: types::Sdflags) -> io::Result<()> {
        unsafe { yanix::socket::shutdown(self.as_raw_fd(), yanix::socket::ShutdownMode::try_from(how)? ) }
    }
}