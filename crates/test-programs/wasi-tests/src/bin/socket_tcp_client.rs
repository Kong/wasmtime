use std::{env, process};

unsafe fn test_socket_tcp_client(port: u16) {
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
    wasi::sock_connect(fd, &mut addr as *mut wasi::Addr)
        .expect("cannot connect to localhost");

    let mut send_content = String::from("Hello World");
    let sent = wasi::sock_send(fd, send_content.as_mut_ptr(), send_content.len(), 0)
        .expect("cannot send content");

    let recv_content = &mut [0u8; 64];
    let recv = wasi::sock_recv(fd, recv_content.as_mut_ptr(), recv_content.len(), 0)
        .expect("cannot receive content");
    assert_eq!(send_content.as_bytes(), &recv_content[0..recv], "no equal payloads");

    wasi::sock_close(fd)
        .expect("cannot close socket");
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
        test_socket_tcp_client(arg.parse::<u16>().unwrap());
    }
}
