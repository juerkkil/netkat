use async_std::os::unix::net::{UnixListener, UnixStream};
use clap::Parser;

use crate::{stdio_utils, Args, Result, Socket};

pub async fn run_unix_socket_server(addr: &str) -> Result<()> {
    let args = Args::parse();
    if args.verbose {
        eprintln!("Listening to unix socket at {}", addr)
    }

    let listener = UnixListener::bind(addr).await?;
    let (sock, peer) = listener.accept().await?;
    if args.verbose {
        eprintln!("Got connection from {:?}", peer);
    }
    let write_sock = Socket::UnixSocketStream(sock.clone());
    let read_sock = Socket::UnixSocketStream(sock);
    stdio_utils::run_async_tasks(read_sock, write_sock).await
}

pub async fn run_unix_socket_client(addr: &str) -> Result<()> {
    let args = Args::parse();
    if args.verbose {
        eprintln!("Connecting to Unix socket at {}", addr);
    }
    let sock = UnixStream::connect(addr).await?;
    if args.verbose {
        eprintln!("Successfully connected to {}", addr)
    }
    let write_sock = Socket::UnixSocketStream(sock.clone());
    stdio_utils::run_async_tasks(Socket::UnixSocketStream(sock), write_sock).await
}
