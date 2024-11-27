use async_std::io;
use async_std::{
    net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs, UdpSocket},
    os::unix::net::UnixStream,
};

use clap::Parser;

use futures::AsyncWriteExt;

use crate::{stdio_utils, Args, Result};

// UDP Connection is not really a thing since UDP is a stateless protocol,
// but in our case UdpSocket +  SocketAddr -pair represents a "connection"
// analogous to TcpStream
pub struct UdpConnection {
    pub socket: UdpSocket,
    pub peer: SocketAddr,
}

pub enum Socket {
    TCP(TcpStream),
    UDP(UdpConnection),
    UnixSocketStream(UnixStream),
    //    UnixSocketDatagram(UnixDatagram),
}

pub async fn run_tcp_server(bind_addr: &str, bind_port: u16) -> Result<()> {
    let args = Args::parse();
    let serveraddr = format!("{}:{}", bind_addr, bind_port);
    if args.verbose {
        eprintln!("Listening to TCP socket at {}", serveraddr)
    }
    let listener = TcpListener::bind(serveraddr).await?;
    match listener.accept().await {
        Ok((stream, addr)) => {
            if args.verbose {
                eprintln!("Client connected from {}", addr)
            }
            let write_sock = Socket::TCP(stream.clone());
            let read_sock = Socket::TCP(stream);
            stdio_utils::run_async_tasks(read_sock, write_sock).await?
        }
        Err(_) => {}
    }
    Ok(())
}

pub async fn run_tcp_client(hostname: &str, target_port: u16, timeout: Option<u64>) -> Result<()> {
    let args = Args::parse();
    let target = format!("{}:{}", hostname, target_port);
    if args.verbose {
        eprintln!("Connecting to {}", target);
    }
    let socket_addr = get_socket_address(&target).await?;

    // A bit of dirty hack again, connect_timeout not implemented in async_std::net::TcpStream, thus we first
    // create just a "normal" sync TcpStream and convert it into async.
    let stream = match timeout {
        Some(timeout) => {
            // async_std::net::TcpStream does not implement connect_timeout() so at first we init just standard
            // TcpStream and convert it asynchronous
            let sync_stream = std::net::TcpStream::connect_timeout(
                &socket_addr,
                std::time::Duration::from_secs(timeout),
            )?;
            TcpStream::from(sync_stream)
        }
        None => TcpStream::connect(socket_addr).await?, // No timeout defined
    };
    if args.verbose {
        eprintln!("Succesfully connected to {}", socket_addr);
    }
    let write_sock = Socket::TCP(stream.clone());
    let read_sock = Socket::TCP(stream);
    stdio_utils::run_async_tasks(read_sock, write_sock).await
}

pub async fn run_udp_client(hostname: &str, target_port: u16) -> Result<()> {
    let args = Args::parse();
    let mut bind_addr = "0.0.0.0:0";
    if args.ipv6 {
        bind_addr = "[::]:0"
    }
    let udp_socket = std::net::UdpSocket::bind(bind_addr)?;

    let target = format!("{}:{}", hostname, target_port);
    let server = get_socket_address(&target).await?;

    let cloned_socket = udp_socket.try_clone()?;
    let udp_conn_read = UdpConnection {
        socket: UdpSocket::from(udp_socket),
        peer: server,
    };
    let udp_conn_write = UdpConnection {
        socket: UdpSocket::from(cloned_socket),
        peer: server,
    };
    stdio_utils::run_async_tasks(Socket::UDP(udp_conn_read), Socket::UDP(udp_conn_write)).await
}

pub async fn run_udp_server(bind_addr: &str, bind_port: u16) -> Result<()> {
    let args = Args::parse();
    let serveraddr = format!("{}:{}", bind_addr, bind_port);

    // UDP stuff
    if args.verbose {
        eprintln!("Listening udp socket at {:?}", serveraddr);
    }

    let udp_socket = std::net::UdpSocket::bind(serveraddr)?;
    let mut buf = [0_u8; crate::BUFFER_SIZE];

    // First come first served
    let (bytes, peer) = udp_socket.recv_from(&mut buf)?;
    if args.verbose {
        eprintln!("Peer connected at {:?}", peer)
    }
    io::stdout().write_all(&buf[0..bytes]).await?;
    io::stdout().flush().await?;

    let cloned_socket = udp_socket.try_clone()?;

    let udp_conn_read = UdpConnection {
        socket: UdpSocket::from(udp_socket),
        peer,
    };
    let udp_conn_write = UdpConnection {
        socket: UdpSocket::from(cloned_socket),
        peer,
    };
    stdio_utils::run_async_tasks(Socket::UDP(udp_conn_read), Socket::UDP(udp_conn_write)).await
}

async fn get_socket_address(target: &str) -> Result<SocketAddr> {
    let args = Args::parse();
    let addrs = target.to_socket_addrs().await?;
    for addr in addrs {
        // IP version not defined, so we just return the first SocketAddr
        if !args.ipv6 && !args.ipv4 {
            return Ok(addr);
        }

        if args.ipv4 && addr.is_ipv4() {
            return Ok(addr);
        }

        if args.ipv6 && addr.is_ipv6() {
            return Ok(addr);
        }
    }
    if args.ipv6 {
        return Err("Unable to resolve a valid IPv6 address".into());
    }
    if args.ipv4 {
        return Err("Unable to resolve a valid IPv6 address".into());
    }
    Err("Could not resolve the address".into())
}
