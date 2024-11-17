use async_std::io::{self};
use clap::Parser;
use futures::AsyncWriteExt;
use futures::{future::FutureExt, pin_mut, select};

use async_std::net::{ToSocketAddrs, UdpSocket};

use crate::{std_socket_io, Args, Result, Socket, UdpConnection};

pub async fn run_udp_client(hostname: &str, target_port: u16) -> Result<()> {
    let udp_socket = std::net::UdpSocket::bind("0.0.0.0:0")?;
    let target = format!("{}:{}", hostname, target_port);
    let server = match target.to_socket_addrs().await?.next() {
        Some(server) => server,
        None => return Err("Empty socket address".into()),
    };

    let cloned_socket = udp_socket.try_clone()?;
    let udp_conn_read = UdpConnection {
        socket: UdpSocket::from(udp_socket),
        peer: server,
    };
    let udp_conn_write = UdpConnection {
        socket: UdpSocket::from(cloned_socket),
        peer: server,
    };
    let conn_read = Socket::UDP(udp_conn_read);
    let conn_write = Socket::UDP(udp_conn_write);

    let stdin_task = std_socket_io::stdin_to_socket(conn_write).fuse();
    let stdout_task = std_socket_io::socket_to_stdout(conn_read).fuse();

    pin_mut!(stdin_task, stdout_task);
    select! {
        _res = stdin_task => _res?,
        _res = stdout_task => _res?,
    }
    Ok(())
}

pub async fn run_udp_server(bind_addr: &str, bind_port: u16) -> Result<()> {
    let args = Args::parse();
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

    let udp_conn_read = UdpConnection {
        socket: UdpSocket::from(udp_socket),
        peer: peer,
    };
    let udp_conn_write = UdpConnection {
        socket: UdpSocket::from(cloned_socket),
        peer: peer,
    };
    let conn_read = Socket::UDP(udp_conn_read);
    let conn_write = Socket::UDP(udp_conn_write);

    let stdin_task = std_socket_io::stdin_to_socket(conn_write).fuse();
    let stdout_task = std_socket_io::socket_to_stdout(conn_read).fuse();

    pin_mut!(stdin_task, stdout_task);
    select! {
        _res = stdin_task => _res?,
        _res = stdout_task => _res?,
    }
    return Ok(());
}
