use wasi_tests::sock_addr_local;

unsafe fn test_sock_bind() {
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
                port: 8080,
            }
        },
    };

    let fd = wasi::sock_open(wasi::ADDRESS_FAMILY_INET4, wasi::SOCK_TYPE_SOCKET_STREAM)
        .expect("cannot open socket");
    wasi::sock_bind(fd, &mut addr as *mut wasi::Addr)
        .expect("cannot bind socket");

    // [EINVAL]
    // Socket is already bound to an address and the protocol does not support binding to a new address.
    // Alternatively, socket may have been shut down.
    assert_eq!(
        wasi::sock_bind(fd, &mut addr as *mut wasi::Addr)
            .expect_err("cannot bind socket")
            .raw_error(),
        wasi::ERRNO_INVAL,
        "errno should be ERRNO_INVAL",
    );

    let bound_addr = sock_addr_local(fd);
    match bound_addr.tag {
        wasi::ADDR_TYPE_IP4 => {
            assert!(
                bound_addr.u.ip4.addr.n0 == addr.u.ip4.addr.n0 &&
                    bound_addr.u.ip4.addr.n1 == addr.u.ip4.addr.n1 &&
                    bound_addr.u.ip4.addr.h0 == addr.u.ip4.addr.h0 &&
                    bound_addr.u.ip4.addr.h1 == addr.u.ip4.addr.h1
                , "invalid ip address");
            assert!(bound_addr.u.ip4.port == addr.u.ip4.port, "invalid port number");
        },
        _ => assert!(true, "invalid address type")
    }

    wasi::sock_close(fd)
        .expect("cannot close socket");
}

fn main() {
    // Run the tests.
    unsafe {
        test_sock_bind();
    }
}
