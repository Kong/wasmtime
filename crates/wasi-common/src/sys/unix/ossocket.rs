use std::io;
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
}