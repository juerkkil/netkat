use async_std::io;
use async_std::net::{TcpListener, TcpStream};

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

    /// Verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.listen {
        match args.port {
            Some(1_u16..=u16::MAX) => {
                return async_std::task::block_on(run_server(
                    &args.hostname.unwrap(),
                    args.port.unwrap(),
                ));
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
        return async_std::task::block_on(run_client(&args.hostname.unwrap(), args.port.unwrap()));
    }

    Ok(())
}

async fn run_client(hostname: &str, target_port: u16) -> Result<()> {
    let target = format!("{}:{}", hostname, target_port);
    let mut stream = TcpStream::connect(target).await?;
    run_tasks(&mut stream).await
}

async fn run_server(bind_addr: &str, bind_port: u16) -> Result<()> {
    let args = Args::parse();
    let serveraddr = format!("{}:{}", bind_addr, bind_port);
    if args.verbose {
        eprintln!("Listening to {}", serveraddr)
    }
    let listener = TcpListener::bind(serveraddr).await?;
    match listener.accept().await {
        Ok((mut stream, addr)) => {
            if args.verbose {
                eprintln!("Client connected from {}", addr)
            }
            run_tasks(&mut stream).await?
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

async fn run_tasks(stream: &mut TcpStream) -> Result<()> {
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
