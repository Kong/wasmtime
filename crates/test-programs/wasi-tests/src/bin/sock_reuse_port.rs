use wasi_tests::STDPOOL_FD;

unsafe fn test_socket_reuse_port() {
    let fd = wasi::sock_open(STDPOOL_FD, wasi::ADDRESS_FAMILY_INET4, wasi::SOCK_TYPE_SOCKET_STREAM)
        .expect("cannot open socket");
    wasi::sock_set_reuse_port(fd, 1)
        .expect("cannot reuse port");
    let reuse = wasi::sock_get_reuse_port(fd)
        .expect("cannot retrieve reuse status");

    assert_eq!(reuse, 1, "socket is not in port reuse mode");

    wasi::sock_close(fd)
        .expect("cannot close socket");
}

fn main() {
    // Run the tests.
    unsafe {
        test_socket_reuse_port();
    }
}
