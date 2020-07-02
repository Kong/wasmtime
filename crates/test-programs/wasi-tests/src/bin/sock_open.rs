use wasi_tests::STDIN_FD;

unsafe fn test_sock_open_badf() {
    assert_eq!(
        wasi::sock_open(9, wasi::ADDRESS_FAMILY_INET4, wasi::SOCK_TYPE_SOCKET_STREAM)
            .expect_err("bad file descriptor")
            .raw_error(),
        wasi::ERRNO_BADF,
        "errno should be ERRNO_BADF",
    );
}

unsafe fn test_sock_open_notpool() {
    assert_eq!(
        wasi::sock_open(STDIN_FD, wasi::ADDRESS_FAMILY_INET4, wasi::SOCK_TYPE_SOCKET_STREAM)
            .expect_err("invalid file descriptor")
            .raw_error(),
        wasi::ERRNO_INVAL,
        "errno should be ERRNO_INVAL",
    );
}

fn main() {
    // Run the tests.
    unsafe {
        test_sock_open_badf();
        test_sock_open_notpool();
    }
}
