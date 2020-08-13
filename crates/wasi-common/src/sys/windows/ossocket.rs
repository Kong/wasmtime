use std::io;
use std::convert::TryFrom;
use winx::socket::socket;
use winapi::um::winsock2 as sock;
use winapi::shared::ws2def;
use winapi::shared::ws2ipdef;
use winapi::ctypes;
use crate::wasi::types;

#[derive(Debug)]
pub(crate) struct RawOsSocket(sock::SOCKET);

impl From<types::AddressFamily> for winx::socket::AddressFamily {
    fn from(af: types::AddressFamily) -> Self {
        match af {
            types::AddressFamily::Inet4 => winx::socket::AddressFamily::InetAddr,
            types::AddressFamily::Inet6 => winx::socket::AddressFamily::Inet6Addr
        }
    }
}

impl From<types::SockType> for winx::socket::SockType {
    fn from(t: types::SockType) -> Self {
        match t {
            types::SockType::SocketStream => winx::socket::SockType::Stream,
            types::SockType::SocketDgram => winx::socket::SockType::Datagram
        }
    }
}

impl From<&types::AddrIp4> for winx::socket::InAddr {
    fn from(addr: &types::AddrIp4) -> Self {
        let mut ip: winapi::shared::inaddr::in_addr_S_un = unsafe { std::mem::zeroed() };
        unsafe { *(ip.S_addr_mut()) = (((addr.n0 as u32) << 24) |
            ((addr.n1 as u32) << 16) |
            ((addr.h0 as u32) << 8) |
            ((addr.h1 as u32) << 0)).to_be() };

        winx::socket::InAddr(winapi::shared::inaddr::IN_ADDR { S_un: ip })
    }
}

impl From<winx::socket::InAddr> for types::AddrIp4 {
    fn from(addr: winx::socket::InAddr) -> Self {
        unsafe {
            types::AddrIp4 {
                n0: addr.0.S_un.S_un_b().s_b1,
                n1: addr.0.S_un.S_un_b().s_b2,
                h0: addr.0.S_un.S_un_b().s_b3,
                h1: addr.0.S_un.S_un_b().s_b4
            }
        }
    }
}

impl From<winx::socket::In6Addr> for types::AddrIp6 {
    fn from(addr: winx::socket::In6Addr) -> Self {
        unsafe {
            types::AddrIp6 {
                n0: addr.0.u.Word()[0],
                n1: addr.0.u.Word()[1],
                n2: addr.0.u.Word()[2],
                n3: addr.0.u.Word()[3],
                h0: addr.0.u.Word()[4],
                h1: addr.0.u.Word()[5],
                h2: addr.0.u.Word()[6],
                h3: addr.0.u.Word()[7]
            }
        }
    }
}

impl From<&types::AddrIp6> for winx::socket::In6Addr {
    fn from(addr: &types::AddrIp6) -> Self {
        unsafe {
            let mut ip: winapi::shared::in6addr::in6_addr_u = std::mem::zeroed();
            ip.Word_mut()[0] = addr.n0;
            ip.Word_mut()[1] = addr.n1;
            ip.Word_mut()[2] = addr.n2;
            ip.Word_mut()[3] = addr.n3;
            ip.Word_mut()[4] = addr.h0;
            ip.Word_mut()[5] = addr.h1;
            ip.Word_mut()[6] = addr.h2;
            ip.Word_mut()[7] = addr.h3;

            winx::socket::In6Addr(winapi::shared::in6addr::IN6_ADDR { u: ip })
        }
    }
}

impl From<&types::Addr> for winx::socket::SockAddr {
    fn from(t: &types::Addr) -> Self {
        match t {
            types::Addr::Ip4(addr) => {
                use std::mem::MaybeUninit;
                let mut storage = MaybeUninit::<ws2def::SOCKADDR_STORAGE>::zeroed();
                let ptr = storage.as_mut_ptr() as *mut ws2def::SOCKADDR_IN;
                unsafe {
                    (*ptr).sin_family = winx::socket::AddressFamily::InetAddr as ws2def::ADDRESS_FAMILY;
                    (*ptr).sin_port = addr.port.to_be();
                    (*ptr).sin_addr = winx::socket::InAddr::from(&addr.addr).0;
                };
                winx::socket::SockAddr {
                    storage: unsafe { storage.assume_init() },
                    len: std::mem::size_of::<ws2def::SOCKADDR_IN>() as ctypes::c_int
                }
            },
            types::Addr::Ip6(addr) => {
                use std::mem::MaybeUninit;
                let mut storage = MaybeUninit::<ws2def::SOCKADDR_STORAGE>::zeroed();
                let ptr = storage.as_mut_ptr() as *mut ws2ipdef::SOCKADDR_IN6_LH;
                unsafe {
                    (*ptr).sin6_family = winx::socket::AddressFamily::Inet6Addr as ws2def::ADDRESS_FAMILY;
                    (*ptr).sin6_port = addr.port.to_be();
                    (*ptr).sin6_addr = winx::socket::In6Addr::from(&addr.addr).0;
                };
                winx::socket::SockAddr {
                    storage: unsafe { storage.assume_init() },
                    len: std::mem::size_of::<ws2def::SOCKADDR_IN>() as ctypes::c_int
                }
            }
        }
    }
}

impl TryFrom<&winx::socket::SockAddr> for types::Addr {
    type Error = io::Error;

    fn try_from(t: &winx::socket::SockAddr) -> Result<Self, Self::Error> {
        let storage = &t.storage as *const ws2def::SOCKADDR_STORAGE;
        if t.storage.ss_family as libc::c_int == ws2def::AF_INET {
            let sockaddr_in = storage as *const ws2def::SOCKADDR_IN;

            unsafe {
                Ok(types::Addr::Ip4(types::AddrIp4Port {
                    addr: types::AddrIp4::from(winx::socket::InAddr((*sockaddr_in).sin_addr)),
                    port: u16::from_be((*sockaddr_in).sin_port),
                }))
            }
        } else if t.storage.ss_family as libc::c_int == ws2def::AF_INET6 {
            let sockaddr_in6 = storage as *const ws2ipdef::SOCKADDR_IN6_LH;

            unsafe {
                Ok(types::Addr::Ip6(types::AddrIp6Port {
                    addr: types::AddrIp6::from(winx::socket::In6Addr((*sockaddr_in6).sin6_addr)),
                    port: u16::from_be((*sockaddr_in6).sin6_port),
                }))
            }
        } else {
            Err(io::Error::from_raw_os_error(libc::EINVAL))
        }
    }
}

impl From<types::Riflags> for winx::socket::RecvFlags {
    fn from(flags: types::Riflags) -> Self {
        let mut out = winx::socket::RecvFlags::empty();
        if flags & types::Riflags::RECV_PEEK == types::Riflags::RECV_PEEK {
            out |= winx::socket::RecvFlags::PEEK;
        }
        if flags & types::Riflags::RECV_WAITALL == types::Riflags::RECV_WAITALL {
            out |= winx::socket::RecvFlags::WAIT_ALL;
        }
        out
    }
}

impl TryFrom<types::Sdflags> for winx::socket::ShutdownMode {
    type Error = std::io::Error;

    fn try_from(flags: types::Sdflags) -> io::Result<Self> {
        if (flags & types::Sdflags::RD == types::Sdflags::RD) && (flags & types::Sdflags::WR == types::Sdflags::WR) {
            Ok(winx::socket::ShutdownMode::Both)
        } else if flags & types::Sdflags::RD == types::Sdflags::RD {
            Ok(winx::socket::ShutdownMode::Read)
        } else if flags & types::Sdflags::WR == types::Sdflags::WR {
            Ok(winx::socket::ShutdownMode::Write)
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "bad flag"))
        }
    }
}

impl RawOsSocket {
    /// Tries clone `self`.
    pub(crate) fn try_clone(&self) -> io::Result<Self> {
        Ok( RawOsSocket(self.0))
    }

    pub(crate) fn new(address_family: types::AddressFamily, socket_type: types::SockType) -> io::Result<Self> {
        let address_family = winx::socket::AddressFamily::from(address_family);
        let c_socket_type = winx::socket::SockType::from(socket_type);
        let fd = unsafe { socket(address_family, c_socket_type, None)? };
        Ok( RawOsSocket(fd))
    }

    pub(crate) fn bind(&self, addr: &types::Addr) -> io::Result<()> {
        let addr = winx::socket::SockAddr::from(addr);
        unsafe { winx::socket::bind(self.0, &addr ) }
    }

    pub(crate) fn listen(&self, backlog: u32) -> io::Result<()> {
        unsafe { winx::socket::listen(self.0, backlog as usize) }
    }

    pub(crate) fn accept(&self) -> io::Result<(RawOsSocket, types::Addr)> {
        unsafe {
            let fd_and_addr = winx::socket::accept(self.0)?;
            let fd = RawOsSocket(fd_and_addr.0);
            let addr = types::Addr::try_from(&fd_and_addr.1)?;
            Ok((fd, addr))
        }
    }

    pub(crate) fn connect(&self, addr: &types::Addr) -> io::Result<()> {
        let addr = winx::socket::SockAddr::from(addr);
        unsafe { winx::socket::connect(self.0, &addr ) }
    }

    pub(crate) fn recv(&self, buf: &mut [u8], flags: types::Riflags) -> io::Result<usize> {
        unsafe { winx::socket::recv(self.0, buf, winx::socket::RecvFlags::from(flags) ) }
    }

    pub(crate) fn recvfrom(&self, buf: &mut [u8], flags: types::Riflags) -> io::Result<(usize, types::Addr)> {
        unsafe {
            let (size, addr) = winx::socket::recvfrom(self.0, buf, winx::socket::RecvFlags::from(flags))?;
            Ok((size, types::Addr::try_from(&addr)?))
        }
    }

    pub(crate) fn shutdown(&self, how: types::Sdflags) -> io::Result<()> {
        unsafe { winx::socket::shutdown(self.0, winx::socket::ShutdownMode::try_from(how)? ) }
    }

    pub(crate) fn send(&self, buf: &[u8], _flags: types::Siflags) -> io::Result<usize> {
        unsafe { winx::socket::send(self.0, buf, winx::socket::SendFlags::empty() ) }
    }

    pub(crate) fn sendto(&self, buf: &[u8], addr: &types::Addr, _flags: types::Siflags) -> io::Result<usize> {
        let addr = winx::socket::SockAddr::from(addr);
        unsafe { winx::socket::sendto(self.0, buf, &addr, winx::socket::SendFlags::empty() ) }
    }

    pub(crate) fn addr_local(&self) -> io::Result<types::Addr> {
        unsafe {
            let addr = winx::socket::sock_name(self.0)?;
            types::Addr::try_from(&addr)
        }
    }

    pub(crate) fn addr_remote(&self) -> io::Result<types::Addr> {
        unsafe {
            let addr = winx::socket::peer_name(self.0)?;
            types::Addr::try_from(&addr)
        }
    }

    pub(crate) fn set_reuse_addr(&self, reuse: bool) -> io::Result<()> {
        unsafe {
            winx::socket::set_reuse_addr(self.0, reuse)
        }
    }

    pub(crate) fn get_reuse_addr(&self) -> io::Result<bool> {
        unsafe {
            winx::socket::get_reuse_addr(self.0)
        }
    }

    pub(crate) fn set_reuse_port(&self, reuse: bool) -> io::Result<()> {
        unsafe {
            winx::socket::set_reuse_port(self.0, reuse)
        }
    }

    pub(crate) fn get_reuse_port(&self) -> io::Result<bool> {
        unsafe {
            winx::socket::get_reuse_port(self.0)
        }
    }

    pub(crate) fn set_recv_buf_size(&self, size: types::Size) -> io::Result<()> {
        unsafe {
            winx::socket::set_recv_buf_size(self.0, size)
        }
    }

    pub(crate) fn get_recv_buf_size(&self) -> io::Result<types::Size> {
        unsafe {
            winx::socket::get_recv_buf_size(self.0)
        }
    }

    pub(crate) fn set_send_buf_size(&self, size: types::Size) -> io::Result<()> {
        unsafe {
            winx::socket::set_send_buf_size(self.0, size)
        }
    }

    pub(crate) fn get_send_buf_size(&self) -> io::Result<types::Size> {
        unsafe {
            winx::socket::get_send_buf_size(self.0)
        }
    }
}
