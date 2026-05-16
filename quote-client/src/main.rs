use clap::Parser;
use socket2::{Domain, Protocol, Socket, TcpKeepalive, Type};

use std::{
    io::{self, BufRead, BufReader, Write, stdin, stdout},
    net::{SocketAddr, TcpStream},
    path::PathBuf,
    thread,
    time::Duration,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short = 'a', long, default_value = "127.0.0.1:7878")]
    addr_tcp: SocketAddr,
    #[arg(short = 'u', long, default_value = "127.0.0.1:7879")]
    addr_udp: SocketAddr,
    #[arg(short = 'f', long, default_value = "data/chosen_tickers.txt")]
    ticker_lsit_path: PathBuf,
}

enum ConnectionResult {
    Lost,
    Exit,
}

fn main() {
    let args = Args::parse();

    //TODO: Understand how it is possible to go though multiple lines, because server will
    //reply to us using mutliple lines and not just one.

    loop {
        match connect_tcp(&args.addr_tcp) {
            Ok(tcp_stream) => {
                println!("Connected to the server!");
                match handle_connection(tcp_stream) {
                    ConnectionResult::Exit => break,
                    ConnectionResult::Lost => {
                        eprintln!("Failed to connec to the serer, retrying in 2s...");
                        thread::sleep(Duration::from_secs(2));
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to connec to the server {}, retrying in 2s...", e);
                thread::sleep(Duration::from_secs(2));
            }
        }
    }
}

fn connect_tcp(addr: &SocketAddr) -> io::Result<TcpStream> {
    let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;

    //
    socket.set_keepalive(true)?;

    // This is the conditional execution of code.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        use std::time::Duration;

        socket.set_tcp_keepalive(
            &TcpKeepalive::new()
                .with_interval(Duration::from_secs(1))
                .with_time(Duration::from_secs(5)),
        )?;
    }

    // We can write socket.connect(&addr.clone().into())
    // But this would mean we are doing the clone when a cheaper copy is
    // available
    socket.connect(&(*addr).into())?;
    let stream: TcpStream = socket.into();

    // Even though to read from the stream we are using the BufReader wrapper
    // we are still essentially reading from the TcpStream
    // and this means, that we are dealing with the read timeout of this socket
    // and we need to control it so it does not block everything forever
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;

    Ok(stream)
}

// Not that we are passing the ownership of the tcp_stream inside the handle_connection function
fn handle_connection(tcp_stream: TcpStream) -> ConnectionResult {
    let mut reader = match tcp_stream.try_clone() {
        Ok(s) => BufReader::new(s),
        Err(e) => {
            eprintln!("Failed to clone the tcp_stream {}", e);
            return ConnectionResult::Lost;
        }
    };

    // Fist we are reading the initial greeting message
    for _ in 0..1 {
        let mut greeting = String::new();
        if let Err(e) = reader.read_line(&mut greeting) {
            eprintln!("Failed to read server's greeting: {}", e);
            return ConnectionResult::Lost;
        }
        print!("{}", greeting);
    }

    // Only then we start an infinite loop where we send the commands to the Server
    loop {
        print!("ypfinancialmarket>");
        if let Err(e) = stdout().flush() {
            eprintln!("Fail to flush the stdout: {}", e);
            return ConnectionResult::Lost;
        }

        let mut input = String::new();
        if let Err(e) = stdin().read_line(&mut input) {
            eprintln!("Failed to read server's greeting: {}", e);
            return ConnectionResult::Lost;
        }
        let trimmed_input = input.trim();

        if trimmed_input.is_empty() {
            continue;
        }

        //TODO: Understand how it is possible to go though multiple lines, because server will
        //reply to us using mutliple lines and not just one.

        // We send the EXIT command to the server, now it is time to close the client
        // Because in this case we are ending prematurely and are not calling the send_command()
        // function. We never recieved the farewell message and never displayed it,
        // We need to show something in here. This means on the server side there is
        // no need to create anything fancy inside the EXIT, it would be unreachable code.
        if trimmed_input.eq_ignore_ascii_case("EXIT") {
            return ConnectionResult::Exit;
        }

        //TODO: inrduce the logs for both this application and the server
        if trimmed_input.eq_ignore_ascii_case("PING") {
            match send_ping(&tcp_stream, &mut reader) {
                Ok(latency) => println!("PONG (latency: {}ms)", latency),
                Err(e) => {
                    eprintln!("ERROR: while recieving a ping from the server {}", e);
                    return ConnectionResult::Lost;
                }
            }
            // we do not want to go inside the send_command section
            continue;
        }

        match send_command(&tcp_stream, &mut reader, trimmed_input) {
            Ok(output) => {
                println!("{}", output);
            }
            Err(e) => {
                eprintln!(
                    "ERROR: while sending or recieving a command from the server {}",
                    e
                );
                return ConnectionResult::Lost;
            }
        }
    }
}

fn send_ping(stream: &TcpStream, reader: &mut BufReader<TcpStream>) -> io::Result<u64> {
    let mut stream = stream;

    todo!()
}

fn send_command(
    stream: &TcpStream,
    reader: &mut BufReader<TcpStream>,
    command: &str,
) -> io::Result<String> {
    let mut stream = stream;

    stream.write_all(command.as_bytes())?;
    stream.write_all(b"\n")?;
    stream.flush()?;

    let mut respnse = String::new();
    let size_response = reader.read_line(&mut respnse)?;

    if size_response == 0 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Server closed connection",
        ));
    }

    Ok(respnse)
}
