unsafe fn test_socket_reuse_addr() {
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
    wasi::sock_set_reuse_addr(fd, 1)
        .expect("cannot reuse address");
    let reuse = wasi::sock_get_reuse_addr(fd)
        .expect("cannot retrieve reuse status");

    assert_eq!(reuse, 1, "socket is not in reuse mode");

    wasi::sock_close(fd)
        .expect("cannot close socket");
}

fn main() {
    // Run the tests.
    unsafe {
        test_socket_reuse_addr();
    }
}
