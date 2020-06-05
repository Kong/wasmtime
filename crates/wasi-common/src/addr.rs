use crate::wasi::types;
use std::io;
use std::vec;
use std::net::{ToSocketAddrs, SocketAddr};

pub trait ToWasiSocketAddrs<'a> {
    type Iter: Iterator<Item=types::Addr<'a>>;

    fn to_wasi_socket_addrs(&self) -> io::Result<Self::Iter>;
}

impl<'a> ToWasiSocketAddrs<'a> for (&str, u16) {
    type Iter = vec::IntoIter<types::Addr<'a>>;
    fn to_wasi_socket_addrs(&self) -> io::Result<vec::IntoIter<types::Addr<'a>>> {
        let addresses = self.to_socket_addrs()?;
        let v: Vec<_> = addresses.into_iter().map(|addr| {
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
        Ok(v.into_iter())
    }
}