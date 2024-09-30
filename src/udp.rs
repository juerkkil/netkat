use async_std::io::{self};
use clap::Parser;
use futures::AsyncWriteExt;
use futures::{future::FutureExt, pin_mut, select};

use async_std::net::{ToSocketAddrs, UdpSocket};

pub async fn run_udp_client(hostname: &str, target_port: u16) -> crate::Result<()> {
    let udp_socket = std::net::UdpSocket::bind("127.0.0.1:0")?;
    let cloned_socket = udp_socket.try_clone()?;
    let async_socket = UdpSocket::from(udp_socket);
    let async_clone = UdpSocket::from(cloned_socket);
    let target = format!("{}:{}", hostname, target_port);
    let server = target.to_socket_addrs().await.unwrap().next();

    let stdin_task = crate::stdin_to_udpsocket(async_socket, server.expect("fail")).fuse();
    let stdout_task = crate::udpsocket_to_stdout(async_clone).fuse();

    pin_mut!(stdin_task, stdout_task);
    select! {
        _res = stdin_task => _res?,
        _res = stdout_task => _res?,
    }
    Ok(())
}

pub async fn run_udp_server(bind_addr: &str, bind_port: u16) -> crate::Result<()> {
    let args = crate::Args::parse();
    let serveraddr = format!("{}:{}", bind_addr, bind_port);

    // UDP stuff
    if args.verbose {
        eprintln!("Listening udp socket at {:?}", serveraddr);
    }

    // Some dirty hacks here, since async_std::net::UdpSocket doesn't implement try_clone(),
    // we'll first create non-async UDP socket, clone it and turn into async sockets once
    // we have an active peer.
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
    let async_socket = UdpSocket::from(udp_socket);
    let async_clone = UdpSocket::from(cloned_socket);

    let stdin_task = crate::stdin_to_udpsocket(async_socket, peer).fuse();
    let stdout_task = crate::udpsocket_to_stdout(async_clone).fuse();

    pin_mut!(stdin_task, stdout_task);
    select! {
        _res = stdin_task => _res?,
        _res = stdout_task => _res?,
    }
    return Ok(());
}