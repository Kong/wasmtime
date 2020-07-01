use std::{thread, time};
use std::net::{TcpListener, SocketAddrV4, SocketAddrV6, Ipv4Addr, Ipv6Addr, TcpStream};
use std::io::{Read, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct EchoTcpServer {
    port: u16,
    stop_flag: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>
}

impl EchoTcpServer {
    pub fn start() -> anyhow::Result<EchoTcpServer> {
        let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
        let listener = TcpListener::bind(addr).unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        listener.set_nonblocking(true)?;

        let shared_stop = stop.clone();
        let local_addr = listener.local_addr()?;
        let thread = thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        let mut read = [0; 1028];
                        match stream.read(&mut read) {
                            Ok(n) => {
                                if n == 0 {
                                    // connection was closed
                                    break;
                                }
                                stream.write(&read[0..n]).unwrap();
                                stream.flush().unwrap();
                            }
                            Err(err) => {
                                panic!(err);
                            }
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(time::Duration::from_millis(10));
                        if shared_stop.load(Ordering::Relaxed) {
                            break;
                        } else {
                            continue;
                        }
                    }
                    Err(e) => panic!("encountered IO error: {}", e),
                }
            }
        });

        Ok(EchoTcpServer {
            port: local_addr.port(),
            stop_flag: stop.clone(),
            thread: Some(thread)
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Drop for EchoTcpServer {
    fn drop(&mut self) {
        println!("stopping echo tcp server");
        self.stop_flag.swap(true, Ordering::Relaxed);
        self.thread.take().unwrap().join().expect("cannot join thread");
    }
}

pub struct EchoTcpClient {
    port: u16,
    stop_flag: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>
}

impl EchoTcpClient {
    fn free_port() -> Option<u16> {
        for port in 15000..25000 {
            let ipv4 = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
            let ipv6 = SocketAddrV6::new(Ipv6Addr::LOCALHOST, port, 0, 0);

            if TcpListener::bind(ipv4).is_ok() && TcpListener::bind(ipv6).is_ok() {
                return Some(port);
            }
        }

        None
    }

    pub fn start() -> anyhow::Result<EchoTcpClient> {
        let port = EchoTcpClient::free_port().unwrap();
        let stop = Arc::new(AtomicBool::new(false));

        let shared_stop = stop.clone();
        let thread = thread::spawn(move || {
            let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);

            while !shared_stop.load(Ordering::Relaxed) {
                match TcpStream::connect(addr) {
                    Ok(mut stream) => {
                        let mut read = [0; 1028];
                        match stream.read(&mut read) {
                            Ok(n) => {
                                if n == 0 {
                                    // connection was closed
                                    break;
                                }
                                stream.write(&read[0..n]).unwrap();
                                stream.flush().unwrap();
                            }
                            Err(err) => {
                                panic!(err);
                            }
                        }
                    },
                    Err(_) => {}
                }

                thread::sleep(time::Duration::from_millis(10));
            }
        });

        Ok(EchoTcpClient {
            port,
            stop_flag: stop.clone(),
            thread: Some(thread)
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Drop for EchoTcpClient {
    fn drop(&mut self) {
        println!("stopping echo tcp client");
        self.stop_flag.swap(true, Ordering::Relaxed);
        self.thread.take().unwrap().join().expect("cannot join thread");
        println!("done");
    }
}