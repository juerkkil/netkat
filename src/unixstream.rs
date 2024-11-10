use async_std::os::unix::net::{UnixListener, UnixStream};
use clap::Parser;
use futures::{future::FutureExt, pin_mut, select};

use crate::{std_socket_io, Args, Result, Socket};

pub async fn run_unix_socket_server(fh: &str) -> Result<()> {
    let args = Args::parse();
    if args.verbose {
        eprintln!("Listening to unix socket at {}", fh)
    }

    let listener = UnixListener::bind(fh).await?;
    match listener.accept().await {
        Ok((stream, _addr)) => run_unixstream_tasks(stream).await?,
        Err(_) => {}
    }
    Ok(())
}

async fn run_unixstream_tasks(stream: UnixStream) -> Result<()> {
    let socket = Socket::UnixSocketStream(stream.clone());
    let socket2 = Socket::UnixSocketStream(stream);
    let stdin_task = std_socket_io::stdin_to_stream(socket2).fuse();
    let stdout_task = std_socket_io::socket_to_stdout(socket).fuse();
    pin_mut!(stdin_task, stdout_task);
    select! {
        _res = stdin_task => _res ,
        _res = stdout_task => _res,
    }
}
