use async_std::os::unix::net::{UnixListener, UnixStream};
use clap::Parser;
use futures::{future::FutureExt, pin_mut, select};

use crate::{std_socket_io, Args, Result, Socket};

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
    let sock_write = Socket::UnixSocketStream(sock.clone());
    let sock_read = Socket::UnixSocketStream(sock);
    run_unixstream_tasks(sock_read, sock_write).await
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
    let wsock = Socket::UnixSocketStream(sock.clone());
    run_unixstream_tasks(Socket::UnixSocketStream(sock), wsock).await
}

async fn run_unixstream_tasks(rsock: Socket, wsock: Socket) -> Result<()> {
    let stdin_task = std_socket_io::stdin_to_socket(wsock).fuse();
    let stdout_task = std_socket_io::socket_to_stdout(rsock).fuse();
    pin_mut!(stdin_task, stdout_task);
    select! {
        _res = stdin_task => _res,
        _res = stdout_task => _res,
    }
}
