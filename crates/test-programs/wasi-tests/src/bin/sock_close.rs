use wasi_tests::sock_addr_local;

unsafe fn test_sock_close() {
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
    wasi::sock_close(fd)
        .expect("cannot close socket");

    assert_eq!(
        wasi::sock_set_reuse_addr(fd, 1)
            .expect_err("bad file descriptor")
            .raw_error(),
        wasi::ERRNO_BADF,
        "errno should be ERRNO_BADF",
    );
}

fn main() {
    // Run the tests.
    unsafe {
        test_sock_close();
    }
}
