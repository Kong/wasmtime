use more_asserts::assert_ge;
use std::{cmp::min, mem, slice, str};

const BUF_LEN: usize = 256;

// Smaller than a single type::Addr. Size of type::Addr is 24.
const SMALL_BUF_LEN: usize = 20;

struct Addrs<'a> {
    buf: &'a [u8],
}

impl<'a> Addrs<'a> {
    fn from_slice(buf: &'a [u8]) -> Self {
        Self { buf }
    }
}

impl<'a> Iterator for Addrs<'a> {
    type Item = wasi::Addr;

    fn next(&mut self) -> Option<wasi::Addr> {
        unsafe {
            if self.buf.is_empty() {
                return None;
            }

            let addr_ptr = self.buf.as_ptr() as *const wasi::Addr;
            let addr = addr_ptr.read_unaligned();
            let delta = mem::size_of_val(&addr);

            self.buf = &self.buf[delta..];

            Some(addr)
        }
    }
}

unsafe fn exec_addr_resolve(host: &str) -> Vec<wasi::Addr> {
    let mut buf: [u8; BUF_LEN] = [0; BUF_LEN];
    let bufused =
        wasi::addr_resolve(host, 0, buf.as_mut_ptr(), BUF_LEN).expect("failed addr_resolve");

    let sl = slice::from_raw_parts(buf.as_ptr(), min(BUF_LEN, bufused));
    let addresses: Vec<_> = Addrs::from_slice(sl).collect();
    addresses
}

unsafe fn test_addr_resolve() {
    let addresses = exec_addr_resolve("localhost");
    assert_ge!(addresses.len(), 1, "localhost should at least resolve to one IP address");
    for addr in addresses {
        match addr.tag {
            wasi::ADDR_TYPE_IP4 => {
                assert!(
                    addr.u.ip4.addr.n0 == 127 &&
                        addr.u.ip4.addr.n1 == 0 &&
                        addr.u.ip4.addr.h0 == 0 &&
                        addr.u.ip4.addr.h1 == 1
                    , "invalid ip address");
            }
            wasi::ADDR_TYPE_IP6 => {
                assert!(
                    addr.u.ip6.addr.n0 == 0 &&
                        addr.u.ip6.addr.n1 == 0 &&
                        addr.u.ip6.addr.n2 == 0 &&
                        addr.u.ip6.addr.n3 == 0 &&
                        addr.u.ip6.addr.h0 == 0 &&
                        addr.u.ip6.addr.h1 == 0 &&
                        addr.u.ip6.addr.h2 == 0 &&
                        addr.u.ip6.addr.h3 == 1
                    , "invalid ip address");
            }
            _ => assert!(true, "invalid address type")
        }
    }
}

unsafe fn test_addr_resolve_no_overflow() {
    let mut buf: [u8; SMALL_BUF_LEN] = [0; SMALL_BUF_LEN];
    let bufused =
        wasi::addr_resolve("localhost", 0, buf.as_mut_ptr(), SMALL_BUF_LEN).expect("failed addr_resolve");
    assert_eq!(bufused, 0, "most likely we overflow the buffer");
}

fn main() {
    // Run the tests.
    unsafe {
        test_addr_resolve();
        test_addr_resolve_no_overflow();
    }
}
