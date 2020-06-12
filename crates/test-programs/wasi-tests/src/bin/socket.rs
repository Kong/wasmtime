unsafe fn test_socket_server() {
    let mut addr = wasi::Addr {
        tag: wasi::ADDR_TYPE_IP4,
        u: wasi::AddrU {
            ip4: wasi::AddrIp4Port {
                addr: wasi::AddrIp4 {
                    n0: 0,
                    n1: 0,
                    h0: 0,
                    h1: 0,
                },
                port: 5000,
            }
        },
    };

    let fd = wasi::sock_open(wasi::ADDRESS_FAMILY_INET4, wasi::SOCK_TYPE_SOCKET_STREAM).unwrap();
    wasi::sock_bind(fd, &mut addr as *mut wasi::Addr);
    wasi::sock_listen(fd, 10);

    //loop {
        let childfd = wasi::sock_accept(fd).unwrap();

        let mut contents = String::from("Hello World");
        let sent = wasi::sock_send(childfd, contents.as_mut_ptr(), contents.len(), 0);

        wasi::sock_shutdown(childfd, wasi::SDFLAGS_RD | wasi::SDFLAGS_WR);
    //}

    wasi::sock_shutdown(fd, wasi::SDFLAGS_RD | wasi::SDFLAGS_WR);
}

unsafe fn test_socket_client() {
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
    wasi::sock_recv(fd, contents.as_mut_ptr(), contents.len(), wasi::RIFLAGS_RECV_WAITALL);

    wasi::sock_shutdown(fd, wasi::SDFLAGS_RD | wasi::SDFLAGS_WR);
}

fn main() {
    // Run the tests.
    unsafe {
        test_socket_client();
    }
}
