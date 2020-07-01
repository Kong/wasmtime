use std::{env, process};
use wasi_tests::sock_addr_local;
use wasi_tests::sock_addr_remote;

unsafe fn test_socket_tcp_server(port: u16) {
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
                port,
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

    let mut send_content = String::from("Hello World3");
    let sent = wasi::sock_send(childfd, send_content.as_mut_ptr(), send_content.len(), 0)
        .expect("cannot send content");
    println!("sent {} bytes", sent);

    let recv_content = &mut [0u8; 64];
    let recv = wasi::sock_recv(childfd, recv_content.as_mut_ptr(), recv_content.len(), 0)
        .expect("cannot receive content");
    println!("recv {} bytes", recv);
    assert_eq!(send_content.as_bytes(), &recv_content[0..recv], "no equal payloads");

    wasi::sock_close(childfd)
        .expect("cannot shutdown child socket");
    //}

    wasi::sock_close(fd)
        .expect("cannot shutdown socket");
}

fn main() {
    let mut args = env::args();
    let prog = args.next().unwrap();
    let arg = if let Some(arg) = args.next() {
        arg
    } else {
        eprintln!("usage: {} <server port>", prog);
        process::exit(1);
    };

    unsafe {
        test_socket_tcp_server(arg.parse::<u16>().unwrap());
    }
}
