use wasi_tests::STDPOOL_FD;

unsafe fn test_socket_reuse_addr() {
    let fd = wasi::sock_open(STDPOOL_FD, wasi::ADDRESS_FAMILY_INET4, wasi::SOCK_TYPE_SOCKET_STREAM)
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
