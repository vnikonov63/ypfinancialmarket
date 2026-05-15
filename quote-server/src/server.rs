use std::{
    collections::HashSet,
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    sync::{Arc, Mutex, atomic::AtomicBool},
    thread,
};

use crate::stock::StockMarket;
use crate::udpsender::StockMarketSenderUDP;

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
                    //TODO: make sure the user provides udp:://127.0.0.1:<port number>
                    // for now it would just be 127.0.0.1:<port number>
                    Some("STREAM") => {
                        let address = parts.next();
                        let tickers = parts.next();

                        if let (Some(address), Some(tickers_str)) = (address, tickers) {
                            let m = stock_market.lock().unwrap();
                            let mut tickers: HashSet<String> = HashSet::new();
                            let mut error: Option<String> = None;

                            for ticker in tickers_str.split(',') {
                                let ticker = ticker.trim();

                                match m.stocks.get(ticker) {
                                    Some(_) => {
                                        tickers.insert(ticker.to_string());
                                    }
                                    None => {
                                        error = Some(format!(
                                            "ERROR: Unable to subscribe to broadcast. One of the tickers: {} is invalid\n",
                                            ticker
                                        ));
                                        break;
                                    }
                                }
                            }
                            drop(m);

                            if let Some(e) = error {
                                e
                            } else {
                                let stop = Arc::new(AtomicBool::new(false));
                                // NOTE: port 0 mean the OS should pick the port for us.
                                let sender = StockMarketSenderUDP::new(
                                    "0.0.0.0:0",
                                    tickers,
                                    Arc::clone(&stop),
                                    Arc::clone(&stock_market),
                                )
                                .unwrap();
                                let target = address.to_string();
                                thread::spawn(move || {
                                    if let Err(e) = sender.start_broadcasting(&target, 1000) {
                                        eprintln!("UDP broadcast error: {}", e);
                                    }
                                });
                                "Subscribed to stream\n".to_string()
                            }
                        } else {
                            "ERROR: usage STREAM udp://127.0.0.1:<port_number> <ticker-1>,<ticker-2>,...,<ticker-n>\n".to_string()
                        }
                    }
                    Some("GET") => {
                        if let Some(ticker_str) = parts.next() {
                            let ticker_str = ticker_str.trim();
                            let m = stock_market.lock().unwrap();
                            match m.stocks.get(ticker_str) {
                                Some(stock) => stock.to_string(),
                                None => "ERROR: ticker not found\n".to_string(),
                            }
                        } else {
                            "ERROR: usage GET <ticker>\n".to_string()
                        }
                    }
                    Some("GET_MANY") => {
                        if let Some(tickers_str) = parts.next() {
                            let m = stock_market.lock().unwrap();
                            let mut result = String::new();
                            for ticker in tickers_str.split(',') {
                                let ticker = ticker.trim();

                                match m.stocks.get(ticker) {
                                    Some(stock) => {
                                        result.push_str(&stock.to_string());
                                        result.push('\n');
                                    }
                                    None => {
                                        result.push_str(&format!(
                                            "ERROR: ticker {ticker} not found\n"
                                        ));
                                    }
                                }
                            }

                            result
                        } else {
                            "ERROR: usage GET_MANY <ticker-1>,<ticker-2>,...,<ticker-n>\n"
                                .to_string()
                        }
                    }
                    Some("GET_TOTAL_VOLUME") => {
                        let m = stock_market.lock().unwrap();
                        format!("{}\n", m.total_volume)
                    }
                    Some("LIST") => {
                        if let Some(str) = parts.next() {
                            "ERROR: usage LIST\n".to_string()
                        } else {
                            let m = stock_market.lock().unwrap();
                            let mut result = String::new();
                            for stock in m.stocks.values() {
                                result.push_str(&stock.to_string());
                                result.push('\n');
                            }

                            result
                        }
                    }
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
