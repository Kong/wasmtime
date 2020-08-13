use std::io::{Error, Result};
use std::ptr;
use bitflags::bitflags;
use winapi::shared::ws2def;
use winapi::um::winsock2 as sock;
use winapi::shared::minwindef::DWORD;
use winapi::ctypes;
use std::sync::Once;

fn init() {
    static INIT: Once = Once::new();

    INIT.call_once(|| unsafe {
        let mut data: sock::WSADATA = std::mem::zeroed();
        let ret = sock::WSAStartup(0x202, // version 2.2
                                   &mut data);
        assert_eq!(ret, 0);
    });
}

trait IsSocketError {
    fn is_socket_error(&self) -> bool;
}

impl IsSocketError for i32 {
    fn is_socket_error(&self) -> bool {
        *self == sock::SOCKET_ERROR
    }
}

fn from_socket_error<T: IsSocketError>(t: T) -> Result<T> {
    if t.is_socket_error() {
        Err(Error::from_raw_os_error(unsafe {
            sock::WSAGetLastError()
        }))
    } else {
        Ok(t)
    }
}

trait IsValidSocket {
    fn is_valid_socket(&self) -> bool;
}

fn from_valid_socket<T: IsValidSocket>(t: T) -> Result<T> {
    if t.is_valid_socket() {
        Ok(t)
    } else {
        Err(Error::from_raw_os_error(unsafe { sock::WSAGetLastError() }))
    }
}

macro_rules! impl_is_valid_socket {
    ($($t:ident)*) => ($(impl IsValidSocket for $t {
        fn is_valid_socket(&self) -> bool {
            *self != sock::INVALID_SOCKET
        }
    })*)
}

impl_is_valid_socket! { usize }

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AddressFamily {
    InetAddr = ws2def::AF_INET,
    Inet6Addr = ws2def::AF_INET6,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SockProtocol {
    Tcp = ws2def::IPPROTO_TCP as i32,
    Udp = ws2def::IPPROTO_UDP as i32,
}

#[derive(Debug, Clone, Copy)]
#[repr(i32)]
pub enum SockType {
    Stream = ws2def::SOCK_STREAM,
    Datagram = ws2def::SOCK_DGRAM,
    SeqPacket = ws2def::SOCK_SEQPACKET,
    Raw = ws2def::SOCK_RAW,
    Rdm = ws2def::SOCK_RDM,
}

#[derive(Clone, Copy)]
pub struct InAddr(pub winapi::shared::inaddr::IN_ADDR);

#[derive(Clone, Copy)]
pub struct In6Addr(pub winapi::shared::in6addr::IN6_ADDR);

#[derive(Clone, Copy)]
pub struct SockAddr {
    pub storage: ws2def::SOCKADDR_STORAGE,
    pub len: ctypes::c_int,
}

impl SockAddr {
    fn as_ptr(&self) -> *const ws2def::SOCKADDR  {
        &self.storage as *const _ as *const _
    }

    fn len(&self) -> ctypes::c_int {
        self.len
    }
}

bitflags! {
    pub struct RecvFlags: ctypes::c_int {
        const OOB = 0x1;
        const PEEK = 0x2;
        const WAIT_ALL = 0x4;
    }
}

bitflags! {
    pub struct SendFlags: ctypes::c_int {
        const OOB = 0x1;
        const DONTROUTE = 0x4;
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(i32)]
pub enum ShutdownMode {
    Write = 0x01,
    Read = 0x00,
    Both = 0x02,
}

const WSA_FLAG_OVERLAPPED: DWORD = 0x01;

pub unsafe fn socket(af: AddressFamily, t: SockType, p: Option<SockProtocol>) -> Result<sock::SOCKET> {
    let protocol: ctypes::c_int = match p {
        None => 0,
        Some(p) => p as ctypes::c_int,
    };

    init();

    from_valid_socket( sock::WSASocketW(
        af as ctypes::c_int,
        t as ctypes::c_int,
        protocol,
        ptr::null_mut(),
        0,
        WSA_FLAG_OVERLAPPED,
    ) )
}

pub unsafe fn listen(fd: sock::SOCKET, backlog: usize) -> Result<()> {
    from_socket_error(sock::listen(fd, backlog as ctypes::c_int) )?;
    Ok(())
}


pub unsafe fn accept(fd: sock::SOCKET) -> Result<(sock::SOCKET, SockAddr)> {
    let mut storage: ws2def::SOCKADDR_STORAGE = std::mem::zeroed();
    let mut len = std::mem::size_of_val(&storage) as ctypes::c_int;

    let fd = from_valid_socket( sock::accept(fd, &mut storage as *mut _ as *mut _, &mut len) )?;

    Ok((fd, SockAddr { storage, len }))
}

pub unsafe fn bind(fd: sock::SOCKET, addr: &SockAddr) -> Result<()> {
    from_socket_error(sock::bind(fd, addr.as_ptr(), addr.len()))?;
    Ok(())
}

pub unsafe fn connect(fd: sock::SOCKET, addr: &SockAddr) -> Result<()> {
    from_socket_error(sock::connect(fd, addr.as_ptr(), addr.len()))?;
    Ok(())
}

pub unsafe fn recv(fd: sock::SOCKET, buf: &mut [u8], flags: RecvFlags) -> Result<usize> {
    let bufused = from_socket_error(sock::recv(
        fd,
        buf.as_mut_ptr() as *mut ctypes::c_char,
        buf.len() as ctypes::c_int,
        flags.bits,
    ))?;
    Ok(bufused as usize)
}

pub unsafe fn recvfrom(fd: sock::SOCKET, buf: &mut [u8], flags: RecvFlags) -> Result<(usize, SockAddr)> {
    let mut storage: ws2def::SOCKADDR_STORAGE = std::mem::zeroed();
    let mut len = std::mem::size_of_val(&storage) as ctypes::c_int;

    let bufused = from_socket_error(sock::recvfrom(
        fd,
        buf.as_mut_ptr() as *mut ctypes::c_char,
        buf.len() as ctypes::c_int,
        flags.bits,
        &mut storage as *mut _ as *mut _,
        &mut len,
    ))?;
    Ok((bufused as usize, SockAddr { storage, len }))
}

pub unsafe fn send(fd: sock::SOCKET, buf: &[u8], flags: SendFlags) -> Result<usize> {
    let bufused = from_socket_error(sock::send(
        fd,
        buf.as_ptr() as *const ctypes::c_char,
        buf.len() as ctypes::c_int,
        flags.bits,
    ))?;
    Ok(bufused as usize)
}

pub unsafe fn sendto(fd: sock::SOCKET, buf: &[u8], addr: &SockAddr, flags: SendFlags) -> Result<usize> {
    let bufused = from_socket_error(sock::sendto(
        fd,
        buf.as_ptr() as *const ctypes::c_char,
        buf.len() as ctypes::c_int,
        flags.bits,
        addr.as_ptr(),
        addr.len(),
    ))?;
    Ok(bufused as usize)
}

pub unsafe fn sock_name(fd: sock::SOCKET) -> Result<SockAddr> {
    let mut storage: ws2def::SOCKADDR_STORAGE = std::mem::zeroed();
    let mut len = std::mem::size_of_val(&storage) as ctypes::c_int;

    from_socket_error(sock::getsockname(
        fd,
        &mut storage as *mut _ as *mut _,
        &mut len
    ))?;

    Ok(SockAddr { storage, len })
}

pub unsafe fn peer_name(fd: sock::SOCKET) -> Result<SockAddr> {
    let mut storage: ws2def::SOCKADDR_STORAGE = std::mem::zeroed();
    let mut len = std::mem::size_of_val(&storage) as ctypes::c_int;

    from_socket_error(sock::getpeername(
        fd,
        &mut storage as *mut _ as *mut _,
        &mut len
    ))?;

    Ok(SockAddr { storage, len })
}


pub unsafe fn shutdown(fd: sock::SOCKET, how: ShutdownMode) -> Result<()> {
    from_socket_error(sock::shutdown(fd, how as ctypes::c_int))?;
    Ok(())
}

pub unsafe fn set_reuse_addr(fd: sock::SOCKET, reuse: bool) -> Result<()> {
    let reuse = &(reuse as ctypes::c_int) as *const ctypes::c_int as *const ctypes::c_char;
    let size = std::mem::size_of::<ctypes::c_int>() as ctypes::c_int;
    from_socket_error( sock::setsockopt(fd, ws2def::SOL_SOCKET, ws2def::SO_REUSEADDR, reuse, size) )?;
    Ok(())
}

pub unsafe fn get_reuse_addr(fd: sock::SOCKET) -> Result<bool> {
    let mut reuse: ctypes::c_int = std::mem::zeroed();
    let mut len = std::mem::size_of::<ctypes::c_int>() as ctypes::c_int;
    from_socket_error( sock::getsockopt(fd, ws2def::SOL_SOCKET, ws2def::SO_REUSEADDR, &mut reuse as *mut _ as *mut _, &mut len) )?;
    Ok(reuse != 0)
}

pub unsafe fn set_reuse_port(_fd: sock::SOCKET, _reuse: bool) -> Result<()> {
    /* let reuse = &(reuse as ctypes::c_int) as *const ctypes::c_int as *const ctypes::c_void;
    let size = std::mem::size_of::<ctypes::c_int>() as ctypes::c_int;
    from_socket_error( sock::setsockopt(fd, ws2def::SOL_SOCKET, ws2def::SO_REUSEPORT, reuse, size) ) */
    Ok(())
}

pub unsafe fn get_reuse_port(_fd: sock::SOCKET) -> Result<bool> {
    /* let mut reuse: ctypes::c_int = std::mem::zeroed();
    let mut len = std::mem::size_of::<ctypes::c_int>() as ctypes::c_int;
    from_socket_error( sock::getsockopt(fd, ws2def::SOL_SOCKET, ws2def::SO_REUSEPORT, &mut reuse as *mut _ as *mut _, &mut len) )?;
    Ok(reuse != 0) */
    Ok(false)
}

pub unsafe fn set_recv_buf_size(fd: sock::SOCKET, buf_size: u32) -> Result<()> {
    let buf_size = &(buf_size as ctypes::c_int) as *const ctypes::c_int as *const ctypes::c_char;
    let size = std::mem::size_of::<ctypes::c_int>() as ctypes::c_int;
    from_socket_error( sock::setsockopt(fd, ws2def::SOL_SOCKET, ws2def::SO_RCVBUF, buf_size, size) )?;
    Ok(())
}

pub unsafe fn get_recv_buf_size(fd: sock::SOCKET) -> Result<u32> {
    let mut buf_size: ctypes::c_int = std::mem::zeroed();
    let mut len = std::mem::size_of::<ctypes::c_int>() as ctypes::c_int;
    from_socket_error( sock::getsockopt(fd, ws2def::SOL_SOCKET, ws2def::SO_RCVBUF, &mut buf_size as *mut _ as *mut _, &mut len) )?;
    Ok(buf_size as u32)
}

pub unsafe fn set_send_buf_size(fd: sock::SOCKET, buf_size: u32) -> Result<()> {
    let buf_size = &(buf_size as ctypes::c_int) as *const ctypes::c_int as *const ctypes::c_char;
    let size = std::mem::size_of::<ctypes::c_int>() as ctypes::c_int;
    from_socket_error( sock::setsockopt(fd, ws2def::SOL_SOCKET, ws2def::SO_SNDBUF, buf_size, size) )?;
    Ok(())
}

pub unsafe fn get_send_buf_size(fd: sock::SOCKET) -> Result<u32> {
    let mut buf_size: ctypes::c_int = std::mem::zeroed();
    let mut len = std::mem::size_of::<ctypes::c_int>() as ctypes::c_int;
    from_socket_error( sock::getsockopt(fd, ws2def::SOL_SOCKET, ws2def::SO_SNDBUF, &mut buf_size as *mut _ as *mut _, &mut len) )?;
    Ok(buf_size as u32)
}

pub unsafe fn get_socket_type(fd: sock::SOCKET) -> Result<SockType> {
    use std::mem::MaybeUninit;
    let mut buffer = MaybeUninit::<SockType>::zeroed().assume_init();
    let mut out_len = std::mem::size_of::<SockType>() as ctypes::c_int;
    from_socket_error(sock::getsockopt(
        fd,
        ws2def::SOL_SOCKET,
        ws2def::SO_TYPE,
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
