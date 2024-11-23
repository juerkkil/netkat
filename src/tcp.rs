use async_std::net::{TcpListener, TcpStream};

use clap::Parser;

use crate::{get_socket_address, std_socket_io, Args, Result, Socket};

// // pub async fn run_server(bind_addr: &str, bind_port: Option<u16>) -> Result<()> {}

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
            std_socket_io::run_async_tasks(read_sock, write_sock).await?
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
    std_socket_io::run_async_tasks(read_sock, write_sock).await
}
