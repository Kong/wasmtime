const BUF_LEN: usize = 8192;

unsafe fn test_socket_recv_buf_size() {
    let fd = wasi::sock_open(wasi::ADDRESS_FAMILY_INET4, wasi::SOCK_TYPE_SOCKET_STREAM)
        .expect("cannot open socket");
    wasi::sock_set_recv_buf_size(fd, BUF_LEN)
        .expect("cannot set recv buf size");
    let size = wasi::sock_get_recv_buf_size(fd)
        .expect("cannot retrieve recv buf size");

    assert_eq!(size, BUF_LEN, "wrong buffer size");

    wasi::sock_close(fd)
        .expect("cannot close socket");
}

fn main() {
    // Run the tests.
    unsafe {
        test_socket_recv_buf_size();
    }
}
