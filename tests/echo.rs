extern crate coio;

use std::io::{Read, Write};

use coio::Scheduler;
use coio::net::{TcpListener, TcpStream, UdpSocket, Shutdown};

#[cfg(unix)]
use coio::net::{UnixStream, UnixListener};

#[test]
fn test_tcp_echo() {

    Scheduler::new().run(move|| {
        // Listener
        let listen_fut = Scheduler::spawn(move|| {
            let acceptor = TcpListener::bind("127.0.0.1:6789").unwrap();
            let (mut stream, _) = acceptor.accept().unwrap();

            let mut buf = [0u8; 1024];
            while let Ok(len) = stream.read(&mut buf) {
                if len == 0 {
                    // EOF
                    break;
                }

                stream.write_all(&buf[..len])
                      .and_then(|_| stream.flush()).unwrap();
            }
        });

        let sender_fut = Scheduler::spawn(move|| {
            let mut stream = TcpStream::connect("127.0.0.1:6789").unwrap();
            stream.write_all(b"abcdefg")
                  .and_then(|_| stream.flush()).unwrap();

            let mut buf = [0u8; 1024];
            let len = stream.read(&mut buf).unwrap();

            stream.shutdown(Shutdown::Both).unwrap();

            assert_eq!(&buf[..len], b"abcdefg");
        });

        listen_fut.join().unwrap();
        sender_fut.join().unwrap();
    }).unwrap();

}

#[test]
fn test_udp_echo() {

    Scheduler::new().run(move|| {
        // Listener
        let listen_fut = Scheduler::spawn(move|| {
            let acceptor = UdpSocket::bind("127.0.0.1:6789").unwrap();

            let mut buf = [0u8; 1024];
            let (len, addr) = acceptor.recv_from(&mut buf).unwrap();
            acceptor.send_to(&buf[..len], addr).unwrap();
        });

        let sender_fut = Scheduler::spawn(move|| {
            let sender = UdpSocket::bind("127.0.0.1:6780").unwrap();

            let mut buf = [0u8; 1024];
            sender.send_to(b"abcdefg", "127.0.0.1:6789").unwrap();
            let (len, _) = sender.recv_from(&mut buf).unwrap();

            assert_eq!(&buf[..len], b"abcdefg");
        });

        listen_fut.join().unwrap();
        sender_fut.join().unwrap();
    }).unwrap();

}

#[cfg(unix)]
#[test]
fn test_unix_socket_echo() {
    Scheduler::new().run(move|| {
        // Listener
        let listen_fut = Scheduler::spawn(move|| {
            let acceptor = UnixListener::bind("127.0.0.1:6789").unwrap();
            let mut stream = acceptor.accept().unwrap();

            let mut buf = [0u8; 1024];
            let len = stream.read(&mut buf).unwrap();
            stream.write_all(&buf[..len])
                  .and_then(|_| stream.flush()).unwrap();
        });

        let sender_fut = Scheduler::spawn(move|| {
            let mut stream = UnixStream::connect("127.0.0.1:6789").unwrap();
            stream.write_all(b"abcdefg")
                  .and_then(|_| stream.flush()).unwrap();

            let mut buf = [0u8; 1024];
            let len = stream.read(&mut buf).unwrap();

            assert_eq!(&buf[..len], b"abcdefg");
        });

        listen_fut.join().unwrap();
        sender_fut.join().unwrap();
    }).unwrap();
}
