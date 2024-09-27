use async_std::io::{self};
use async_std::net::{SocketAddr, UdpSocket};
use futures::{AsyncReadExt, AsyncWriteExt};

use clap::Parser;

mod stdio;
mod tcp;
mod udp;

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
                    return async_std::task::block_on(udp::run_udp_server(
                        &args.hostname.unwrap(),
                        args.port.unwrap(),
                    ));
                } else {
                    return async_std::task::block_on(tcp::run_tcp_server(
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
            return async_std::task::block_on(udp::run_udp_client(
                &args.hostname.unwrap(),
                args.port.unwrap(),
            ));
        } else {
            return async_std::task::block_on(tcp::run_tcp_client(
                &args.hostname.unwrap(),
                args.port.unwrap(),
            ));
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
