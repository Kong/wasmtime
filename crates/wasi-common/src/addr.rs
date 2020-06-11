use std::cell::Cell;
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs, Ipv4Addr, Ipv6Addr};
use std::ops::Range;
use std::vec;

use ipnet::IpNet;

use crate::handle::HandleRights;
use crate::wasi::Result;
use crate::wasi::{types, RightsExt};
use std::rc::Rc;

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

pub(crate) struct AddressPoolTable {
    pools: Vec<Rc<Box<dyn AddressPool>>>
}

impl AddressPoolTable {
    pub(crate) fn new() -> AddressPoolTable {
        AddressPoolTable {
            pools: vec![]
        }
    }

    pub(crate) fn insert(&mut self, pool: Box<dyn AddressPool>) -> &mut Self {
        self.pools.push(Rc::new(pool));
        self
    }

    pub(crate) fn get_pool(&self, addr: &types::Addr) -> Option<Rc<Box<dyn AddressPool>>> {
        let addr = SocketAddr::from(addr);
        for pool in &self.pools {
            if pool.contains(&addr ) {
                return Some(pool.clone());
            }
        }
        None
    }

    pub(crate) fn resolve(&self, host: &str, port: u16) -> Result<Vec<types::Addr>> {
        let addresses = (host, port).to_socket_addrs()?;
        let filtered = addresses.into_iter().filter(|addr| {
            for pool in &self.pools {
                if pool.contains(addr) {
                    return true;
                }
            }
            false
        });
        let v = filtered
            .map(|addr| types::Addr::from(addr))
            .collect();
        Ok(v)
    }
}

pub(crate) trait AddressPool {
    fn get_rights(&self) -> HandleRights {
        HandleRights::empty()
    }
    fn contains(&self, _addr: &SocketAddr) -> bool { false }
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

struct RangedAddressPool {
    rights: Cell<HandleRights>,
    cidr: IpNet,
    ports: Range<u16>,
}

impl AddressPool for FixedAddressPool {
    fn get_rights(&self) -> HandleRights {
        self.rights.get()
    }
    fn contains(&self, addr: &SocketAddr) -> bool {
        self.addrs.contains(addr)
    }
}

impl AddressPool for RangedAddressPool {
    fn get_rights(&self) -> HandleRights {
        self.rights.get()
    }
    fn contains(&self, addr: &SocketAddr) -> bool {
        self.cidr.contains(&addr.ip()) && self.ports.contains(&addr.port())
    }
}