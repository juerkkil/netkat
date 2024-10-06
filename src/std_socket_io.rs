use async_std::io::{self};
use async_std::net::{SocketAddr, TcpStream, UdpSocket};
use futures::{AsyncReadExt, AsyncWriteExt};

use clap::Parser;

use crate::{Args, Result, Socket};

pub async fn stdin_to_stream(mut stream: TcpStream) -> Result<()> {
    let mut stdin = io::stdin();
    let _res = io::copy(&mut stdin, &mut stream).await?;
    Ok(())
}

pub async fn stdin_to_udpsocket(socket: UdpSocket, peer: SocketAddr) -> Result<()> {
    let args = Args::parse();
    let mut buf = [0u8; crate::BUFFER_SIZE];

    loop {
        let read_bytes = io::stdin().read(&mut buf).await?;
        let bytes_sent = match read_bytes {
            1_usize..=usize::MAX => socket.send_to(&buf[0..read_bytes], peer).await,
            _ => break,
        };
        match bytes_sent {
            Ok(1_usize..=usize::MAX) => {}
            Err(_err) => {
                if args.verbose {
                    eprintln!("Connection failed: {:?}", _err);
                }
                break;
            }
            _ => break,
        }
    }
    Ok(())
}

// Generic implementation for udp/tcp to avoid duplicate code
// Socket `socket` is an enum over async TcpStream and UdpSocket
pub async fn socket_to_stdout(mut socket: Socket) -> Result<()> {
    let mut stdout = io::stdout();
    let mut buf = [0u8; crate::BUFFER_SIZE];
    loop {
        let (bytes_read, _peer) = match socket {
            Socket::TCP(ref mut stream) => (stream.read(&mut buf).await?, stream.peer_addr()?),
            Socket::UDP(ref udp_socket) => udp_socket.recv_from(&mut buf).await?,
        };
        match bytes_read {
            1_usize..=usize::MAX => {
                stdout.write_all(&buf[0..bytes_read]).await?;
                stdout.flush().await?;
            }
            _ => break,
        }
    }
    Ok(())
}
