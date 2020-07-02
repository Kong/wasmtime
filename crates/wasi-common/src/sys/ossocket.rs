use std::cell::Cell;
use crate::handle::{HandleRights, Handle};
use crate::wasi::types;
use super::sys_impl::ossocket::RawOsSocket;
use std::any::Any;
use std::io;
use crate::wasi;

impl From<types::SockType> for types::Filetype {
    fn from(t: types::SockType) -> Self {
        match t {
            types::SockType::SocketStream => types::Filetype::SocketStream,
            types::SockType::SocketDgram => types::Filetype::SocketDgram
        }
    }
}

pub(crate) struct OsSocket {
    socket_type: types::SockType,
    rights: Cell<HandleRights>,
    handle: RawOsSocket,
    addr_pool: Option<Box<dyn Handle>>
}

impl OsSocket {
    pub(crate) fn new(address_family: types::AddressFamily, socket_type: types::SockType, addr_pool: Box<dyn Handle>) -> io::Result<Self> {
        let raw_socket = RawOsSocket::new(address_family, socket_type)?;
        let rights = Cell::new( HandleRights::from_base( types::Rights::empty() ) );
        Ok(Self {
            socket_type,
            rights,
            handle: raw_socket,
            addr_pool: Some(addr_pool)
        })
    }
}

impl Handle for OsSocket {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn try_clone(&self) -> io::Result<Box<dyn Handle>> {
        let socket_type = self.socket_type;
        let handle = self.handle.try_clone()?;
        let rights = self.rights.clone();
        let addr_pool = if self.addr_pool.is_some() { Some(self.addr_pool.as_ref().unwrap().try_clone()?) } else { None };
        Ok(Box::new(Self {
            socket_type,
            rights,
            handle,
            addr_pool
        }))
    }



    fn get_file_type(&self) -> types::Filetype {
        types::Filetype::from(self.socket_type)
    }

    fn get_rights(&self) -> HandleRights {
        self.rights.get()
    }

    fn set_rights(&self, new_rights: HandleRights) {
        self.rights.set(new_rights)
    }

    fn sock_connect(&self, addr: &types::Addr) -> wasi::Result<()> {
        if !(self.addr_pool.is_some() && self.addr_pool.as_ref().unwrap().addr_pool_contains(addr)?) {
            Err(types::Errno::Notcapable)
        } else {
            self.handle.connect(addr)?;
            Ok(())
        }
    }

    fn sock_bind(&self, addr: &types::Addr) -> wasi::Result<()> {
        if !(self.addr_pool.is_some() && self.addr_pool.as_ref().unwrap().addr_pool_contains(addr)?) {
            Err(types::Errno::Notcapable)
        } else {
            self.handle.bind(addr)?;
            Ok(())
        }
    }

    fn sock_listen(&self, backlog: u32) -> wasi::Result<()> {
        self.handle.listen(backlog)?;
        Ok(())
    }

    fn sock_accept(&self) -> wasi::Result<Box<dyn Handle>> {
        let (raw_socket, _addr) = self.handle.accept()?;
        let rights = Cell::new( self.get_rights() );
        let socket = OsSocket {
            socket_type: self.socket_type,
            rights,
            handle: raw_socket,
            addr_pool: None
        };
        Ok(socket.try_clone()?)
    }

    fn sock_shutdown(&self, how: types::Sdflags) -> wasi::Result<()> {
        self.handle.shutdown(how)?;
        Ok(())
    }

    fn sock_recv(&self, buf: &mut [u8], flags: types::Riflags) -> wasi::Result<usize> {
        let size = self.handle.recv(buf, flags)?;
        Ok(size)
    }

    fn sock_recv_from(&self, buf: &mut [u8], flags: types::Riflags) -> wasi::Result<(usize, types::Addr)> {
        let (size, addr) = self.handle.recvfrom(buf, flags)?;
        Ok((size, addr))
    }

    fn sock_send(&self, buf: &[u8], flags: types::Siflags) -> wasi::Result<usize> {
        let size = self.handle.send(buf, flags)?;
        Ok(size)
    }

    fn sock_send_to(&self, buf: &[u8], addr: &types::Addr, flags: types::Siflags) -> wasi::Result<usize> {
        if !(self.addr_pool.is_some() && self.addr_pool.as_ref().unwrap().addr_pool_contains(addr)?) {
            Err(types::Errno::Notcapable)
        } else {
            let size = self.handle.sendto(buf, addr, flags)?;
            Ok(size)
        }
    }

    fn sock_addr_local(&self) -> wasi::Result<types::Addr> {
        let addr = self.handle.addr_local()?;
        Ok(addr)
    }

    fn sock_addr_remote(&self) -> wasi::Result<types::Addr> {
        let addr = self.handle.addr_remote()?;
        Ok(addr)
    }

    fn sock_set_reuse_addr(&self, reuse: bool) -> wasi::Result<()> {
        self.handle.set_reuse_addr(reuse)?;
        Ok(())
    }

    fn sock_get_reuse_addr(&self) -> wasi::Result<bool> {
        let reuse = self.handle.get_reuse_addr()?;
        Ok(reuse)
    }

    fn sock_set_reuse_port(&self, reuse: bool) -> wasi::Result<()> {
        self.handle.set_reuse_port(reuse)?;
        Ok(())
    }

    fn sock_get_reuse_port(&self) -> wasi::Result<bool> {
        let reuse = self.handle.get_reuse_port()?;
        Ok(reuse)
    }

    fn sock_set_recv_buf_size(&self, size: types::Size) -> wasi::Result<()> {
        self.handle.set_recv_buf_size(size)?;
        Ok(())
    }

    fn sock_get_recv_buf_size(&self) -> wasi::Result<types::Size> {
        let size = self.handle.get_recv_buf_size()?;
        Ok(size)
    }

    fn sock_set_send_buf_size(&self, size: types::Size) -> wasi::Result<()> {
        self.handle.set_send_buf_size(size)?;
        Ok(())
    }

    fn sock_get_send_buf_size(&self) -> wasi::Result<types::Size> {
        let size = self.handle.get_send_buf_size()?;
        Ok(size)
    }


}