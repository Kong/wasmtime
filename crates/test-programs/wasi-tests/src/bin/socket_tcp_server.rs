use wasi_tests::sock_addr_local;
use wasi_tests::sock_addr_remote;

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
    let local_addr = sock_addr_local(fd);
    match local_addr.tag {
        wasi::ADDR_TYPE_IP4 => {
            assert!(
                local_addr.u.ip4.addr.n0 == 127 &&
                    local_addr.u.ip4.addr.n1 == 0 &&
                    local_addr.u.ip4.addr.h0 == 0 &&
                    local_addr.u.ip4.addr.h1 == 1
                , "invalid ip address");
        }
        _ => assert!(true, "invalid address type")
    }

    wasi::sock_listen(fd, 10)
        .expect("cannot listen on socket");

    //loop {
    let childfd = wasi::sock_accept(fd)
        .expect("unable to accept connection");
    let remote_addr = sock_addr_remote(childfd);
    println!("client connected {}", wasi_tests::PrintableAddr(remote_addr));

    let mut contents = String::from("Hello World from Kong");
    let sent = wasi::sock_send(childfd, contents.as_mut_ptr(), contents.len(), 0)
        .expect("cannot send");
    println!("sent {} bytes", sent);

    wasi::sock_close(childfd)
        .expect("cannot shutdown child socket");
    //}

    wasi::sock_close(fd)
        .expect("cannot shutdown socket");
}

fn main() {
    // Run the tests.
    unsafe {
        test_socket_tcp_server();
    }
}
