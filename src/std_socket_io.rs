use async_std::io::{self};
use async_std::net::{SocketAddr, UdpSocket};
use futures::{AsyncReadExt, AsyncWriteExt};

use crate::{Result, Socket};

pub async fn stdin_to_stream(mut stream: Socket) -> Result<()> {
    let mut stdin = io::stdin();
    let _res = match stream {
        Socket::TCP(ref mut stream) => io::copy(&mut stdin, stream).await?,
        Socket::UDP(ref _udp_socket) => todo!(),
        Socket::UnixSocketStream(ref mut stream) => io::copy(&mut stdin, stream).await?,
        Socket::UnixSocketDatagram(ref _asdf) => todo!(),
    };

    Ok(())
}

pub async fn stdin_to_udpsocket(socket: UdpSocket, peer: SocketAddr) -> Result<()> {
    let mut buf = [0u8; crate::BUFFER_SIZE];

    loop {
        let read_bytes = io::stdin().read(&mut buf).await?;
        let sent_bytes = match read_bytes {
            1_usize..=usize::MAX => socket.send_to(&buf[0..read_bytes], peer).await?,
            _ => break,
        };
        if sent_bytes == 0 {
            // not sure whether this can happen but just in case
            break;
        }
    }
    Ok(())
}

// Generic implementation for udp/tcp to avoid duplicate code
// Socket `socket` is an enum over async TcpStream, UdpSocket and Unix sockets
pub async fn socket_to_stdout(mut socket: Socket) -> Result<()> {
    let mut stdout = io::stdout();
    let mut buf = [0u8; crate::BUFFER_SIZE];
    loop {
        let bytes_read = match socket {
            Socket::TCP(ref mut stream) => stream.read(&mut buf).await?,
            Socket::UDP(ref udp_socket) => udp_socket.recv_from(&mut buf).await?.0,
            Socket::UnixSocketStream(ref mut stream) => stream.read(&mut buf).await?,
            Socket::UnixSocketDatagram(_a) => todo!(),
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
