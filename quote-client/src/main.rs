use clap::Parser;

use std::{
    io::{BufRead, BufReader, Write, stdin, stdout},
    net::{SocketAddr, TcpStream},
    path::PathBuf,
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

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let mut tcp_stream = TcpStream::connect(args.addr_tcp)?;
    let mut reader = BufReader::new(tcp_stream.try_clone()?);

    // Fist we are reading the initial greeting message
    for _ in 0..1 {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        print!("{}", line);
    }

    // Only then we start an infinite loop where we send the commands to the Server
    loop {
        println!("ypfinancialmarket>");
        stdout().flush()?;

        let mut input = String::new();
        stdin().read_line(&mut input)?;
        let trimmed_input = input.trim();

        if trimmed_input.is_empty() {
            continue;
        }

        tcp_stream.write_all(trimmed_input.as_bytes())?;
        tcp_stream.write_all(b"\n")?;
        tcp_stream.flush()?;

        //TODO: Understand how it is possible to go though multiple lines, because server will
        //reply to us using mutliple lines and not just one.
        //
        let mut reply = String::new();
        let reply_byte_size = reader.read_line(&mut reply)?;

        if reply_byte_size == 0 {
            println!("Server closed connection");
            stdout().flush()?;
        }

        print!("{}", reply);

        // We send the EXIT command to the server, now it is time to close the client
        // We recieved the farewell message and displayed it,
        // and there is no need to show one from the client
        if trimmed_input.eq_ignore_ascii_case("EXIT") {
            break;
        }
    }

    Ok(())
}
