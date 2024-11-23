use async_std::io::{self};
use clap::Parser;
use futures::AsyncWriteExt;

use async_std::net::UdpSocket;

use crate::{get_socket_address, std_socket_io, Args, Result, Socket, UdpConnection};

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
    std_socket_io::run_async_tasks(Socket::UDP(udp_conn_read), Socket::UDP(udp_conn_write)).await
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
    std_socket_io::run_async_tasks(Socket::UDP(udp_conn_read), Socket::UDP(udp_conn_write)).await
}
