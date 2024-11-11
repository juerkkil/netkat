use std::net::ToSocketAddrs;

use async_std::net::{TcpListener, TcpStream};

use clap::Parser;
use futures::{future::FutureExt, pin_mut, select};

use crate::{std_socket_io, Args, Result, Socket};

// // pub async fn run_server(bind_addr: &str, bind_port: Option<u16>) -> Result<()> {}

pub async fn run_tcp_server(bind_addr: &str, bind_port: u16) -> Result<()> {
    let args = Args::parse();
    let serveraddr = format!("{}:{}", bind_addr, bind_port);
    if args.verbose {
        eprintln!("Listening to TCP socket at {}", serveraddr)
    }
    let listener = TcpListener::bind(serveraddr).await?;
    match listener.accept().await {
        Ok((mut stream, addr)) => {
            if args.verbose {
                eprintln!("Client connected from {}", addr)
            }
            run_tcpstream_tasks(&mut stream).await?
        }
        Err(_) => {}
    }
    Ok(())
}

pub async fn run_tcp_client(hostname: &str, target_port: u16, timeout: Option<u64>) -> Result<()> {
    let target = format!("{}:{}", hostname, target_port);
    let target_next = match target.to_socket_addrs()?.next() {
        Some(t) => t,
        None => return Err("Empty socket addr".into()),
    };

    // A bit of dirty hack again, connect_timeout not implemented in async_std::net::TcpStream, thus we first
    // create just a "normal" sync TcpStream and convert it into async.
    let mut stream = match timeout {
        Some(timeout) => {
            let sync_stream = std::net::TcpStream::connect_timeout(
                &target_next,
                std::time::Duration::from_secs(timeout),
            )?;
            TcpStream::from(sync_stream)
        }
        None => {
            // Let's go with system's default timeout
            TcpStream::connect(target).await?
        }
    };
    run_tcpstream_tasks(&mut stream).await
}

async fn run_tcpstream_tasks(stream: &mut TcpStream) -> Result<()> {
    let stdin_task = std_socket_io::stdin_to_stream(Socket::TCP(stream.clone())).fuse();
    let socket = Socket::TCP(stream.clone());
    let stdout_task = std_socket_io::socket_to_stdout(socket).fuse();

    // If either of the tasks (reading or writing the stream) is completed, we return without
    // waiting the other one to finish.
    // There is a catch here; if the user sends EOF via stdin, both tasks will be terminated
    // meaning that we might miss some data that's still on the route through network
    pin_mut!(stdin_task, stdout_task);
    select! {
        _res = stdin_task => _res,
        _res = stdout_task => _res,
    }
}
