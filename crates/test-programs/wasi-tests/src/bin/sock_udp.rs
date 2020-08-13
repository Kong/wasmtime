use wasi_tests::STDPOOL_FD;
use std::{env, process};

unsafe fn test_socket_udp_client(port: u16) {
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

    let fd = wasi::sock_open(STDPOOL_FD, wasi::ADDRESS_FAMILY_INET4, wasi::SOCK_TYPE_SOCKET_DGRAM)
        .expect("cannot open socket");

    let mut send_content = String::from("Hello World");
    let sent = wasi::sock_send_to(fd, send_content.as_mut_ptr(), send_content.len(), &mut addr as *mut wasi::Addr, 0)
        .expect("cannot send content");
    assert_eq!(sent, send_content.len(), "wrong number of bytes sent");

    let addr_content = &mut [0u8; 64];
    let recv_content = &mut [0u8; 64];
    let recv = wasi::sock_recv_from(fd, recv_content.as_mut_ptr(), recv_content.len(), addr_content.as_mut_ptr(), addr_content.len(), 0)
        .expect("cannot receive content");

    assert_eq!(recv, send_content.len(), "wrong number of bytes received");
    assert_eq!(send_content.as_bytes(), &recv_content[0..recv], "no equal payloads");

    let addr_ptr = addr_content.as_ptr() as *const wasi::Addr;
    let remote_addr = addr_ptr.read_unaligned();

    assert_eq!(remote_addr.u.ip4.port, port, "wrong port number");

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
        test_socket_udp_client(arg.parse::<u16>().unwrap());
    }
}