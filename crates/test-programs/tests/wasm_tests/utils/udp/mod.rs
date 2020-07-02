use std::{thread, time};
use std::net::{SocketAddrV4, Ipv4Addr, UdpSocket};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct EchoUdpSocket {
    port: u16,
    stop_flag: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>
}

impl EchoUdpSocket {
    pub fn start() -> anyhow::Result<EchoUdpSocket> {
        let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
        let socket = UdpSocket::bind(addr).unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        socket.set_nonblocking(true)?;

        let shared_stop = stop.clone();
        let local_addr = socket.local_addr()?;
        let thread = thread::spawn(move || {
            let mut buf = [0; 1028];

            while !shared_stop.load(Ordering::Relaxed) {
                match socket.recv_from(&mut buf) {
                    Ok((size, send_to)) => {
                        // reply back
                        socket.send_to(&buf[0..size], send_to).unwrap();
                    },
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {},
                    Err(e) => panic!("encountered IO error: {}", e)
                }

                thread::sleep(time::Duration::from_millis(10));
            }
        });

        Ok(EchoUdpSocket {
            port: local_addr.port(),
            stop_flag: stop.clone(),
            thread: Some(thread)
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Drop for EchoUdpSocket {
    fn drop(&mut self) {
        self.stop_flag.swap(true, Ordering::Relaxed);
        self.thread.take().unwrap().join().expect("cannot join thread");
    }
}