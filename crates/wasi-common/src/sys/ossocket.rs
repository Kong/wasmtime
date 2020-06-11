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

#[derive(Debug)]
pub(crate) struct OsSocket {
    socket_type: types::SockType,
    rights: Cell<HandleRights>,
    handle: RawOsSocket,
}

impl OsSocket {
    pub(crate) fn new(address_family: types::AddressFamily, socket_type: types::SockType) -> io::Result<Self> {
        let raw_socket = RawOsSocket::new(address_family, socket_type)?;
        let rights = Cell::new( HandleRights::from_base( types::Rights::empty() ) );
        Ok(Self {
            socket_type,
            rights,
            handle: raw_socket
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
        Ok(Box::new(Self {
            socket_type,
            rights,
            handle,
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
        self.handle.connect(addr)?;
        Ok(())
    }

    fn sock_bind(&self, _addr: &types::Addr) -> wasi::Result<()> {
        unimplemented!()
    }

    fn sock_listen(&self, _backlog: u32) -> wasi::Result<()> {
        unimplemented!()
    }

    fn sock_accept(&self) -> wasi::Result<Box<dyn Handle>> {
        unimplemented!()
    }

    fn sock_shutdown(&self, _how: types::Sdflags) -> wasi::Result<()> {
        unimplemented!()
    }
}