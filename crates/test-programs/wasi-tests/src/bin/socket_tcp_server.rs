unsafe fn test_socket_tcp_server() {
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
    wasi::sock_listen(fd, 10)
        .expect("cannot listen on socket");

    //loop {
    let childfd = wasi::sock_accept(fd)
        .expect("unable to accept connection");

    let mut contents = String::from("Hello World");
    let sent = wasi::sock_send(childfd, contents.as_mut_ptr(), contents.len(), 0);

    wasi::sock_shutdown(childfd, wasi::SDFLAGS_RD | wasi::SDFLAGS_WR);
    //}

    wasi::sock_shutdown(fd, wasi::SDFLAGS_RD | wasi::SDFLAGS_WR);
}

fn main() {
    // Run the tests.
    unsafe {
        test_socket_tcp_server();
    }
}
