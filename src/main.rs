use async_std::io::{self};

use async_std::net::{SocketAddr, ToSocketAddrs};
use async_std::net::{TcpListener, TcpStream, UdpSocket};

use clap::Parser;

use futures::{future::FutureExt, pin_mut, select};
use futures::{AsyncReadExt, AsyncWriteExt};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

const BUFFER_SIZE: usize = 8192;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Hostname (either destination address or the address to bind the listener)
    hostname: Option<String>,

    /// Port - either source or target port depending on mode of operation
    port: Option<u16>,

    /// Listen to incoming connection
    #[arg(short, long, default_value_t = false)]
    listen: bool,

    /// Use UDP instead of TCP
    #[arg(short, long, default_value_t = false)]
    udp: bool,

    /// Verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.listen {
        match args.port {
            Some(1_u16..=u16::MAX) => {
                if args.udp {
                    return async_std::task::block_on(run_tcp_server(
                        &args.hostname.unwrap(),
                        args.port.unwrap(),
                    ));
                } else {
                    return async_std::task::block_on(run_udp_server(
                        &args.hostname.unwrap(),
                        args.port.unwrap(),
                    ));
                }
            }
            Some(_) => {
                eprintln!("Invalid port number");
            }
            None => {
                eprintln!("No source port provided")
            }
        }
    } else {
        /* Check the necessary args for client mode */
        if args.hostname.is_none() {
            eprintln!("No hostname given!");
            return Ok(());
        }
        if args.port.is_none() {
            eprintln!("No port given");
            return Ok(());
        }
        if args.udp {
            return async_std::task::block_on(run_udp_client(
                &args.hostname.unwrap(),
                args.port.unwrap(),
            ));
        } else {
            return async_std::task::block_on(run_tcp_client(
                &args.hostname.unwrap(),
                args.port.unwrap(),
            ));
        }
    }

    Ok(())
}

async fn run_udp_client(hostname: &str, target_port: u16) -> Result<()> {
    let udp_socket = std::net::UdpSocket::bind("127.0.0.1:0")?;
    let cloned_socket = udp_socket.try_clone()?;
    let async_socket = UdpSocket::from(udp_socket);
    let async_clone = UdpSocket::from(cloned_socket);
    let target = format!("{}:{}", hostname, target_port);
    let server = target.to_socket_addrs().unwrap().next().expect("foo");

    let stdin_task = stdin_to_udpsocket(async_socket, server).fuse();
    let stdout_task = udpsocket_to_stdout(async_clone).fuse();

    pin_mut!(stdin_task, stdout_task);
    select! {
        _res = stdin_task => _res?,
        _res = stdout_task => _res?,
    }
    Ok(())
}

async fn run_tcp_client(hostname: &str, target_port: u16) -> Result<()> {
    let target = format!("{}:{}", hostname, target_port);
    let mut stream = TcpStream::connect(target).await?;
    run_tcpstream_tasks(&mut stream).await
}

async fn run_udp_server(bind_addr: &str, bind_port: u16) -> Result<()> {
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
    let mut buf = [0_u8; BUFFER_SIZE];

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

    let stdin_task = stdin_to_udpsocket(async_socket, peer).fuse();
    let stdout_task = udpsocket_to_stdout(async_clone).fuse();

    pin_mut!(stdin_task, stdout_task);
    select! {
        _res = stdin_task => _res?,
        _res = stdout_task => _res?,
    }
    return Ok(());
}

async fn run_tcp_server(bind_addr: &str, bind_port: u16) -> Result<()> {
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

async fn stdin_to_stream(mut stream: TcpStream) -> Result<()> {
    let mut stdin = io::stdin();
    let _res = io::copy(&mut stdin, &mut stream).await?;
    Ok(())
}

async fn stream_to_stdout(mut stream: TcpStream) -> Result<()> {
    let mut stdout = io::stdout();
    // we cloud simply use io::copy to pipe the tcpstream to stdout. However, this doesn't flush
    // stdout unless there is a newline
    //  let _res = io::copy(&mut stream, &mut stdout).await;
    let mut buf = [0u8; BUFFER_SIZE];
    loop {
        let bytes_read = stream.read(&mut buf).await.unwrap();
        match bytes_read {
            1_usize..=usize::MAX => {
                stdout.write_all(&buf[0..bytes_read]).await?;
                stdout.flush().await?;
            }
            _ => {
                // Most likely reached EOF
                stdout.flush().await?;
                break;
            }
        }
    }

    Ok(())
}

async fn stdin_to_udpsocket(socket: UdpSocket, peer: SocketAddr) -> Result<()> {
    let mut buf = [0u8; BUFFER_SIZE];

    loop {
        let read_bytes = io::stdin().read(&mut buf).await.unwrap();
        match read_bytes {
            1_usize..=usize::MAX => {
                socket.send_to(&buf[0..read_bytes], peer).await?;
            }
            _ => break,
        }
    }
    Ok(())
}

async fn udpsocket_to_stdout(socket: UdpSocket) -> Result<()> {
    let mut stdout = io::stdout();
    let mut buf = [0u8; BUFFER_SIZE];
    loop {
        let (bytes, _peer) = socket.recv_from(&mut buf).await?;
        match bytes {
            1_usize..=usize::MAX => {
                stdout.write_all(&buf[0..bytes]).await?;
                stdout.flush().await?;
            }
            _ => {
                // Most likely reached EOF
                stdout.flush().await?;
                break;
            }
        }
    }
    Ok(())
}

async fn run_tcpstream_tasks(stream: &mut TcpStream) -> Result<()> {
    let stdin_task = stdin_to_stream(stream.clone()).fuse();
    let stdout_task = stream_to_stdout(stream.clone()).fuse();

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
