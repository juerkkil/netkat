use async_std::io::{self};
use futures::{future::FutureExt, pin_mut, select};
use futures::{AsyncReadExt, AsyncWriteExt};

use crate::{Result, Socket};

async fn stdin_to_socket(mut sock: Socket) -> Result<()> {
    let mut stdin = io::stdin();
    let _res = match sock {
        Socket::TCP(ref mut stream) => io::copy(&mut stdin, stream).await?,
        Socket::UDP(ref udp_connection) => {
            let mut buf = [0u8; crate::BUFFER_SIZE];
            loop {
                let read_bytes = io::stdin().read(&mut buf).await?;
                let sent_bytes = match read_bytes {
                    1_usize..=usize::MAX => {
                        udp_connection
                            .socket
                            .send_to(&buf[0..read_bytes], udp_connection.peer)
                            .await?
                    }
                    _ => break,
                };
                if sent_bytes == 0 {
                    break;
                }
            }
            0
        }
        Socket::UnixSocketStream(ref mut stream) => io::copy(&mut stdin, stream).await?,
        Socket::UnixSocketDatagram(ref _udp_connection) => todo!(),
    };
    Ok(())
}

// Generic implementation for udp/tcp to avoid duplicate code
// Socket `socket` is an enum over async TcpStream, UdpSocket and Unix sockets
async fn socket_to_stdout(mut socket: Socket) -> Result<()> {
    let mut stdout = io::stdout();
    let mut buf = [0u8; crate::BUFFER_SIZE];
    loop {
        let bytes_read = match socket {
            Socket::TCP(ref mut stream) => stream.read(&mut buf).await?,
            Socket::UDP(ref udp_socket) => udp_socket.socket.recv_from(&mut buf).await?.0,
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

pub async fn run_async_tasks(read_sock: Socket, write_sock: Socket) -> Result<()> {
    let stdin_task = stdin_to_socket(write_sock).fuse();
    let stdout_task = socket_to_stdout(read_sock).fuse();
    pin_mut!(stdin_task, stdout_task);
    select! {
        _res = stdin_task => _res,
        _res = stdout_task => _res,
    }
}
