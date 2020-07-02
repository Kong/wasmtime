use wasi_tests::{sock_addr_local, STDPOOL_FD};

unsafe fn test_sock_connect_not_capable() {
    let mut addr = wasi::Addr {
        tag: wasi::ADDR_TYPE_IP4,
        u: wasi::AddrU {
            ip4: wasi::AddrIp4Port {
                addr: wasi::AddrIp4 {
                    n0: 127,
                    n1: 0,
                    h0: 0,
                    h1: 1,
                },
                port: 9090,
            }
        },
    };

    let fd = wasi::sock_open(STDPOOL_FD, wasi::ADDRESS_FAMILY_INET4, wasi::SOCK_TYPE_SOCKET_STREAM)
        .expect("cannot open socket");

    assert_eq!(
        wasi::sock_connect(fd, &mut addr as *mut wasi::Addr)
            .expect_err("cannot bind socket")
            .raw_error(),
        wasi::ERRNO_NOTCAPABLE,
        "errno should be ERRNO_NOTCAPABLE",
    );

    wasi::sock_close(fd)
        .expect("cannot close socket");
}

fn main() {
    // Run the tests.
    unsafe {
        test_sock_connect_not_capable();
    }
}
