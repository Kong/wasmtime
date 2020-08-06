use std::cell::Cell;
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs, Ipv4Addr, Ipv6Addr};
use std::ops::Range;
use std::vec;
use std::any::Any;
use std::io;

use ipnet::IpNet;

use crate::handle::HandleRights;
use crate::wasi::Result;
use crate::wasi::{types, RightsExt};
use crate::Handle;

impl From<SocketAddrV6> for types::AddrIp6Port {
    fn from(addr: SocketAddrV6) -> Self {
        let segments = addr.ip().segments();
        types::AddrIp6Port {
            addr: types::AddrIp6 {
                n0: segments[0],
                n1: segments[1],
                n2: segments[2],
                n3: segments[3],
                h0: segments[4],
                h1: segments[5],
                h2: segments[6],
                h3: segments[7],
            },
            port: addr.port(),
        }
    }
}

impl From<SocketAddrV4> for types::AddrIp4Port {
    fn from(addr: SocketAddrV4) -> Self {
        let segments = addr.ip().octets();
        types::AddrIp4Port {
            addr: types::AddrIp4 {
                n0: segments[0],
                n1: segments[1],
                h0: segments[2],
                h1: segments[3],
            },
            port: addr.port(),
        }
    }
}

impl From<SocketAddr> for types::Addr {
    fn from(addr: SocketAddr) -> Self {
        match addr {
            SocketAddr::V6(addr) => types::Addr::Ip6(types::AddrIp6Port::from(addr)),
            SocketAddr::V4(addr) => types::Addr::Ip4(types::AddrIp4Port::from(addr))
        }
    }
}

impl From<&types::AddrIp4Port> for SocketAddrV4 {
    fn from(addr: &types::AddrIp4Port) -> Self {
        let port = addr.port;
        let ip = Ipv4Addr::new(addr.addr.n0, addr.addr.n1, addr.addr.h0, addr.addr.h1);
        SocketAddrV4::new(ip, port)
    }
}

impl From<&types::AddrIp6Port> for SocketAddrV6 {
    fn from(addr: &types::AddrIp6Port) -> Self {
        let port = addr.port;
        let ip = Ipv6Addr::new(addr.addr.n0, addr.addr.n1, addr.addr.n2, addr.addr.n3, addr.addr.h0, addr.addr.h1, addr.addr.h2, addr.addr.h3);
        SocketAddrV6::new(ip, port, 0, 0)
    }
}

impl From<&types::Addr> for SocketAddr {
    fn from(addr: &types::Addr) -> Self {
        match addr {
            types::Addr::Ip6(addr) => SocketAddr::V6(SocketAddrV6::from(addr)),
            types::Addr::Ip4(addr) => SocketAddr::V4(SocketAddrV4::from(addr))
        }
    }
}

pub trait AddressFamilyCompatible {
    fn accepts(&self, addr: &types::Addr) -> bool;
}

impl AddressFamilyCompatible for types::AddressFamily {
    fn accepts(&self, addr: &types::Addr) -> bool {
        match self {
            types::AddressFamily::Inet4 => {
                match addr {
                    types::Addr::Ip6(_) => false,
                    _ => true
                }
            },
            types::AddressFamily::Inet6 => {
                match addr {
                    types::Addr::Ip4(_) => false,
                    _ => true
                }
            }
        }
    }
}

pub(crate) struct FixedAddressPool {
    rights: Cell<HandleRights>,
    addrs: Vec<SocketAddr>,
}

impl FixedAddressPool {
    pub(crate) fn new(addr: SocketAddr) -> FixedAddressPool {
        FixedAddressPool {
            rights: Cell::new(
                HandleRights::new(
                    types::Rights::address_pool_base(),
                    types::Rights::address_pool_inheriting()
                )
            ),
            addrs: vec![addr],
        }
    }
}

fn addr_pool_resolve(pool: &dyn Handle, host: &str, port: u16) -> Result<Vec<types::Addr>> {
    let addresses = (host, port).to_socket_addrs()?;
    let v = addresses.into_iter()
        .map(|addr| types::Addr::from(addr))
        .filter(|addr| {
            pool.addr_pool_contains(addr).unwrap_or(false)
        })
        .collect();
    Ok(v)
}

impl Handle for FixedAddressPool {
    fn as_any(&self) -> &dyn Any { self }

    fn try_clone(&self) -> io::Result<Box<dyn Handle>> {
        let addrs = self.addrs.clone();
        let rights = self.rights.clone();
        Ok(Box::new(Self {
            rights,
            addrs
        }))
    }



    fn get_file_type(&self) -> types::Filetype {
        types::Filetype::AddressPool
    }

    fn get_rights(&self) -> HandleRights {
        self.rights.get()
    }

    fn set_rights(&self, new_rights: HandleRights) {
        self.rights.set(new_rights)
    }

    fn is_directory(&self) -> bool { false }

    fn is_tty(&self) -> bool { false }

    fn fdstat_get(&self) -> Result<types::Fdflags> { Err(types::Errno::Badf) }


    fn addr_pool_contains(&self, addr: &types::Addr) -> Result<bool> {
        let addr = SocketAddr::from(addr);
        Ok(self.addrs.contains(&addr))
    }

    fn addr_pool_resolve(&self, host: &str, port: u16) -> Result<Vec<types::Addr>> {
        addr_pool_resolve(self, host, port)
    }
}

struct RangedAddressPool {
    rights: Cell<HandleRights>,
    cidr: IpNet,
    ports: Range<u16>,
}

impl Handle for RangedAddressPool {
    fn as_any(&self) -> &dyn Any { self }

    fn try_clone(&self) -> io::Result<Box<dyn Handle>> {
        let cidr = self.cidr.clone();
        let ports = self.ports.clone();
        let rights = self.rights.clone();
        Ok(Box::new(Self {
            rights,
            cidr,
            ports
        }))
    }

    fn get_file_type(&self) -> types::Filetype {
        types::Filetype::AddressPool
    }

    fn get_rights(&self) -> HandleRights {
        self.rights.get()
    }

    fn set_rights(&self, new_rights: HandleRights) {
        self.rights.set(new_rights)
    }

    fn addr_pool_contains(&self, addr: &types::Addr) -> Result<bool> {
        let addr = SocketAddr::from(addr);
        Ok(self.cidr.contains(&addr.ip()) && self.ports.contains(&addr.port()))
    }

    fn addr_pool_resolve(&self, host: &str, port: u16) -> Result<Vec<types::Addr>> {
        addr_pool_resolve(self, host, port)
    }
}