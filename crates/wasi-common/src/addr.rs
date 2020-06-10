use crate::handle::HandleRights;
use crate::wasi::types;
use crate::wasi::Result;
use std::cell::Cell;
use std::net::{SocketAddr, ToSocketAddrs};
use std::ops::Range;
use std::vec;
use ipnet::IpNet;

pub(crate) struct AddressPoolTable {
    pools: Vec<Box<dyn AddressPool>>
}

impl AddressPoolTable {
    pub(crate) fn new() -> AddressPoolTable {
        AddressPoolTable {
            pools: vec![]
        }
    }

    pub(crate) fn insert(&mut self, pool: Box<dyn AddressPool>) -> &mut Self {
        self.pools.push(pool);
        self
    }

    pub(crate) fn resolve(&self, host: &str, port: u16) -> Result<Vec<types::Addr>> {
        let addresses = (host, port).to_socket_addrs()?;
        let filtered = addresses.into_iter().filter(|addr| {
            for pool in &self.pools {
                if pool.contains(addr) {
                    return true
                }
            }
            false
        });
        let v = filtered.map(|addr| {
            match addr {
                SocketAddr::V6(ref addr) => {
                    let segments = addr.ip().segments();
                    types::Addr::Ip6(
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
                        })
                }
                SocketAddr::V4(ref addr) => {
                    let segments = addr.ip().octets();
                    types::Addr::Ip4(
                        types::AddrIp4Port {
                            addr: types::AddrIp4 {
                                n0: segments[0],
                                n1: segments[1],
                                h0: segments[2],
                                h1: segments[3],
                            },
                            port: addr.port(),
                        }
                    )
                }
            }
        }).collect();
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
    addrs: Vec<SocketAddr>
}

impl FixedAddressPool {
    pub(crate) fn new(addr: SocketAddr) -> FixedAddressPool {
        FixedAddressPool {
            rights: Cell::new(HandleRights::from_base(types::Rights::SOCK_CONNECT ) ),
            addrs: vec![addr]
        }
    }
}

struct RangedAddressPool {
    rights: Cell<HandleRights>,
    cidr: IpNet,
    ports: Range<u16>
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