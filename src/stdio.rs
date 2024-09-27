use async_std::io::{self};
use async_std::net::TcpStream;
use futures::{AsyncReadExt, AsyncWriteExt};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub async fn stdin_to_stream(mut stream: TcpStream) -> Result<()> {
    let mut stdin = io::stdin();
    let _res = io::copy(&mut stdin, &mut stream).await?;
    Ok(())
}

pub async fn stream_to_stdout(mut stream: TcpStream) -> Result<()> {
    let mut stdout = io::stdout();
    // we cloud simply use io::copy to pipe the tcpstream to stdout. However, this doesn't flush
    // stdout unless there is a newline
    //  let _res = io::copy(&mut stream, &mut stdout).await;
    let mut buf = [0u8; crate::BUFFER_SIZE];
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
