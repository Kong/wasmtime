use wasi_tests::STDPOOL_FD;

const BUF_LEN: usize = 8192;

unsafe fn test_socket_send_buf_size() {
    let fd = wasi::sock_open(STDPOOL_FD, wasi::ADDRESS_FAMILY_INET4, wasi::SOCK_TYPE_SOCKET_STREAM)
        .expect("cannot open socket");
    wasi::sock_set_send_buf_size(fd, BUF_LEN)
        .expect("cannot set send buf size");
    let size = wasi::sock_get_send_buf_size(fd)
        .expect("cannot retrieve send buf size");

    assert_eq!(size, BUF_LEN, "wrong buffer size");

    wasi::sock_close(fd)
        .expect("cannot close socket");
}

fn main() {
    // Run the tests.
    unsafe {
        test_socket_send_buf_size();
    }
}
