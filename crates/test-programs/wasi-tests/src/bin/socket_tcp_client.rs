unsafe fn test_socket_tcp_client() {
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
    wasi::sock_connect(fd, &mut addr as *mut wasi::Addr)
        .expect("cannot connect to localhost");

    let contents = &mut [0u8; 64];
    wasi::sock_recv(fd, contents.as_mut_ptr(), contents.len(), wasi::RIFLAGS_RECV_WAITALL)
        .expect("cannot receive content");

    wasi::sock_shutdown(fd, wasi::SDFLAGS_RD | wasi::SDFLAGS_WR)
        .expect("cannot shutdown socket");
}

fn main() {
    // Run the tests.
    unsafe {
        test_socket_tcp_client();
    }
}
