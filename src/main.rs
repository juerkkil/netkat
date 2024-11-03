use async_std::net::{TcpStream, UdpSocket};

use clap::Parser;

mod std_socket_io;
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

    /// Timeout in seconds
    #[arg(short, long)]
    timeout: Option<u64>,

    /// Verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

pub enum Socket {
    TCP(TcpStream),
    UDP(UdpSocket),
}

fn main() -> Result<()> {
    let args = Args::parse();
    let hostname = match args.hostname {
        Some(ref hostname) => hostname,
        None => {
            eprintln!("No hostname given!");
            std::process::exit(1);
        }
    };
    let port = match args.port {
        Some(port) if port > 0_u16 && port <= u16::MAX => port,
        None => {
            eprintln!("No port given");
            std::process::exit(1);
        }
        _ => {
            eprintln!("Invalid port number");
            std::process::exit(1);
        }
    };

    let res: Result<()>;
    if args.listen {
        if args.udp {
            res = async_std::task::block_on(udp::run_udp_server(hostname, port));
        } else {
            res = async_std::task::block_on(tcp::run_tcp_server(hostname, port));
        }
    } else {
        if args.udp {
            res = async_std::task::block_on(udp::run_udp_client(hostname, port));
        } else {
            res = async_std::task::block_on(tcp::run_tcp_client(hostname, port, args.timeout));
        }
    }
    if res.is_ok() {
        return Ok(());
    } else {
        eprintln!("Error: {}", res.unwrap_err().to_string());
        std::process::exit(1);
    }
}
