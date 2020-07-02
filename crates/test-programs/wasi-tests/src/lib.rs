use more_asserts::assert_gt;

const BUF_LEN: usize = 20;

// The `wasi` crate version 0.9.0 and beyond, doesn't
// seem to define these constants, so we do it ourselves.
pub const STDIN_FD: wasi::Fd = 0x0;
pub const STDOUT_FD: wasi::Fd = 0x1;
pub const STDERR_FD: wasi::Fd = 0x2;
pub const STDPOOL_FD: wasi::Fd = 0x3;

/// Opens a fresh file descriptor for `path` where `path` should be a preopened
/// directory.
pub fn open_scratch_directory(path: &str) -> Result<wasi::Fd, String> {
    unsafe {
        for i in 3.. {
            let stat = match wasi::fd_prestat_get(i) {
                Ok(s) => s,
                Err(_) => break,
            };
            if stat.tag != wasi::PREOPENTYPE_DIR {
                continue;
            }
            let mut dst = Vec::with_capacity(stat.u.dir.pr_name_len);
            if wasi::fd_prestat_dir_name(i, dst.as_mut_ptr(), dst.capacity()).is_err() {
                continue;
            }
            dst.set_len(stat.u.dir.pr_name_len);
            if dst == path.as_bytes() {
                let (base, inherit) = fd_get_rights(i);
                return Ok(
                    wasi::path_open(i, 0, ".", wasi::OFLAGS_DIRECTORY, base, inherit, 0)
                        .expect("failed to open dir"),
                );
            }
        }

        Err(format!("failed to find scratch dir"))
    }
}

pub unsafe fn create_file(dir_fd: wasi::Fd, filename: &str) {
    let file_fd =
        wasi::path_open(dir_fd, 0, filename, wasi::OFLAGS_CREAT, 0, 0, 0).expect("creating a file");
    assert_gt!(
        file_fd,
        libc::STDERR_FILENO as wasi::Fd,
        "file descriptor range check",
    );
    wasi::fd_close(file_fd).expect("closing a file");
}

// Returns: (rights_base, rights_inheriting)
pub unsafe fn fd_get_rights(fd: wasi::Fd) -> (wasi::Rights, wasi::Rights) {
    let fdstat = wasi::fd_fdstat_get(fd).expect("fd_fdstat_get failed");
    (fdstat.fs_rights_base, fdstat.fs_rights_inheriting)
}

pub unsafe fn drop_rights(fd: wasi::Fd, drop_base: wasi::Rights, drop_inheriting: wasi::Rights) {
    let (current_base, current_inheriting) = fd_get_rights(fd);

    let new_base = current_base & !drop_base;
    let new_inheriting = current_inheriting & !drop_inheriting;

    wasi::fd_fdstat_set_rights(fd, new_base, new_inheriting).expect("dropping fd rights");
}

pub unsafe fn sock_addr_local(fd: u32) -> wasi::Addr {
    let mut buf: [u8; BUF_LEN] = [0; BUF_LEN];
    wasi::sock_addr_local(fd,  buf.as_mut_ptr(), BUF_LEN)
        .expect("unable to obtain local bound address");
    let addr_ptr = buf.as_ptr() as *const wasi::Addr;
    addr_ptr.read_unaligned()
}

pub unsafe fn sock_addr_remote(fd: u32) -> wasi::Addr {
    let mut buf: [u8; BUF_LEN] = [0; BUF_LEN];
    wasi::sock_addr_remote(fd,  buf.as_mut_ptr(), BUF_LEN)
        .expect("unable to obtain local bound address");
    let addr_ptr = buf.as_ptr() as *const wasi::Addr;
    addr_ptr.read_unaligned()
}

pub struct PrintableAddr(pub wasi::Addr);

impl std::fmt::Display for PrintableAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0.tag {
            wasi::ADDR_TYPE_IP4 => {
                unsafe {
                    write!(f, "{}.{}.{}.{}:{}", self.0.u.ip4.addr.n0, self.0.u.ip4.addr.n1, self.0.u.ip4.addr.h0, self.0.u.ip4.addr.h1, self.0.u.ip4.port)
                }
            }
            _ => write!(f, "invalid address type")
        }
    }
}