use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
};

use crate::stock::StockMarket;

pub fn handle_client(stream: TcpStream, stock_market: Arc<Mutex<StockMarket>>) {
    let mut writer = stream.try_clone().expect("failed to clone stream");
    let mut reader = BufReader::new(stream);

    let _ = writer.write_all(b"Welcome to the ypfinancialmarket. Your resource for being up to date with the latest financial information\n");
    let _ = writer.flush();

    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => {
                return;
            }
            Ok(_) => {
                let input = line.trim();
                if input.is_empty() {
                    let _ = writer.flush();
                    continue;
                }
                let mut parts = input.split_whitespace();
                let response = match parts.next() {
                    Some("STREAM") => {
                        let address = parts.next();
                        let tickers = parts.next();

                        if let (Some(address), Some(tickers)) = (address, tickers) {
                            "YOU SEND COMMAND STREAM".to_string()
                        } else {
                            "ERROR: usage STREAM udp://127.0.0.1:<port_number> <tick1>,<tick2>,...,<tick[n]>\n".to_string()
                        }
                    }
                    Some("GET") => "YOU SEND COMMAND GET\n".to_string(),
                    Some("GET_MANY") => "YOU SEND COMMAND GET_MANY\n".to_string(),
                    Some("GET_TOTAL_VOLUME") => {
                        let mut m = stock_market.lock().unwrap();
                        format!("{}\n", m.total_volume)
                    }
                    Some("LIST") => "YOU SEND COMMAND LIST\n".to_string(),
                    Some("PING") => "YOU SEND COMMAND PING\n".to_string(),
                    Some("CONNECTIONS") => "YOU SEND COMMAND CONNECTIONS\n".to_string(),
                    Some("HELP") => "YOU SEND COMMAND HELP\n".to_string(),
                    _ => "ERROR: Unknown command\n".to_string(),
                };

                let _ = writer.write_all(response.as_bytes());
                let _ = writer.flush();
            }
            Err(_) => {
                return;
            }
        }
    }
}
