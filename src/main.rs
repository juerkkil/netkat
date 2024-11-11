use async_std::{
    net::{TcpStream, UdpSocket},
    os::unix::net::{UnixDatagram, UnixStream},
};

use clap::Parser;

mod std_socket_io;
mod tcp;
mod udp;
mod unixstream;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

const BUFFER_SIZE: usize = 8192;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Hostname (either destination address or the address to bind the listener)
    address: Option<String>,

    /// Port - either source or target port depending on mode of operation
    port: Option<u16>,

    /// Listen to incoming connection
    #[arg(short, long, default_value_t = false)]
    listen: bool,

    /// Use UDP instead of TCP
    #[arg(short, long, default_value_t = false)]
    udp: bool,

    /// Timeout in seconds (only TCP)
    #[arg(short, long)]
    timeout: Option<u64>,

    /// Use UNIX domain socket instead of Internet domain socket
    #[arg(short = 'U')]
    unix_socket: bool,

    /// Verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

pub enum Socket {
    TCP(TcpStream),
    UDP(UdpSocket),
    UnixSocketStream(UnixStream),
    UnixSocketDatagram(UnixDatagram),
}

fn main() -> Result<()> {
    let args = Args::parse();
    let address = match args.address {
        Some(ref address) => address,
        None => {
            eprintln!("No address given!");
            std::process::exit(1);
        }
    };
    let port = match args.port {
        Some(port) if port > 0_u16 && port <= u16::MAX => port,
        None => {
            if !args.unix_socket {
                eprintln!("No port given");
                std::process::exit(1);
            }
            0
        }
        _ => {
            eprintln!("Invalid port number");
            std::process::exit(1);
        }
    };

    let res: Result<()>;
    if args.listen {
        if args.udp {
            res = async_std::task::block_on(udp::run_udp_server(address, port));
        } else if args.unix_socket {
            res = async_std::task::block_on(unixstream::run_unix_socket_server(address));
        } else {
            res = async_std::task::block_on(tcp::run_tcp_server(address, port));
        }
    } else {
        if args.udp {
            res = async_std::task::block_on(udp::run_udp_client(address, port));
        } else if args.unix_socket {
            res = async_std::task::block_on(unixstream::run_unix_socket_client(address));
        } else {
            res = async_std::task::block_on(tcp::run_tcp_client(address, port, args.timeout));
        }
    }
    if res.is_ok() {
        return Ok(());
    } else {
        eprintln!("Error: {}", res.unwrap_err().to_string());
        std::process::exit(1);
    }
}
