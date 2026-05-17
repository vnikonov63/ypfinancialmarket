use std::{
    collections::HashSet,
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::stock::StockMarket;
use crate::udpsender::StockMarketSenderUDP;

pub fn handle_client(stream: TcpStream, stock_market: Arc<Mutex<StockMarket>>) {
    let mut writer = stream.try_clone().expect("failed to clone stream");
    let mut reader = BufReader::new(stream);

    let _ = writer.write_all(b"Welcome to the ypfinancialmarket. Your resource for being up to date with the latest financial information\n");
    let _ = writer.flush();

    let mut udp_stop_flags: Vec<Arc<AtomicBool>> = Vec::new();

    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => {
                break;
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
                                            "1\nERROR: Unable to subscribe to broadcast. One of the tickers: {} is invalid\n",
                                            ticker
                                        ));
                                        break;
                                    }
                                }
                            }
                            // We no longer need m lock at this point, so we should drop it.
                            // We want to minimize the time when a Mutex is locked. We may rely
                            // on the implicit drop when m goes out of scope, but this would
                            // require rewriting the function and creating helpers, increasing the
                            // levels of abstraction that are not good for the learning process.
                            drop(m);

                            if let Some(e) = error {
                                e
                            } else {
                                let stop = Arc::new(AtomicBool::new(false));
                                let stop_clone = Arc::clone(&stop);
                                udp_stop_flags.push(stop_clone);

                                // NOTE: port 0 mean the OS should pick the port for us.
                                let sender = StockMarketSenderUDP::new(
                                    "127.0.0.1:0",
                                    tickers,
                                    Arc::clone(&stop),
                                    Arc::clone(&stock_market),
                                )
                                .unwrap();
                                //TODO: remove this unwrap here, when cleaning the code.
                                let port = sender.socket.local_addr().unwrap().port();
                                let target = address.to_string();
                                thread::spawn(move || {
                                    if let Err(e) =
                                        sender.start_broadcasting_with_ping(&target, 1000)
                                    {
                                        eprintln!("UDP broadcast error: {}", e);
                                    }
                                });
                                format!("2\nSubscribed to stream on port:\n{}\n", port)
                                //"1\nSubscribed to stream\n".to_string()
                            }
                        } else {
                            "1\nERROR: usage STREAM udp://127.0.0.1:<port_number> <ticker-1>,<ticker-2>,...,<ticker-n>\n".to_string()
                        }
                    }
                    Some("GET") => {
                        if let Some(ticker_str) = parts.next() {
                            let ticker_str = ticker_str.trim();
                            let m = stock_market.lock().unwrap();
                            match m.stocks.get(ticker_str) {
                                Some(stock) => format!("1\n{}\n", stock),
                                None => "1\nERROR: ticker not found\n".to_string(),
                            }
                        } else {
                            "1\nERROR: usage GET <ticker>\n".to_string()
                        }
                    }
                    Some("GET_MANY") => {
                        if let Some(tickers_str) = parts.next() {
                            let m = stock_market.lock().unwrap();
                            let tickers: Vec<&str> = tickers_str
                                .split(',')
                                .map(|t| t.trim())
                                .filter(|t| !t.is_empty())
                                .collect();

                            let mut result = String::new();

                            result.push_str(&format!("{}\n", tickers.len()));

                            for ticker in &tickers {
                                match m.stocks.get(*ticker) {
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
                            "1\nERROR: usage GET_MANY <ticker-1>,<ticker-2>,...,<ticker-n>"
                                .to_string()
                        }
                    }
                    Some("GET_TOTAL_VOLUME") => {
                        let m = stock_market.lock().unwrap();
                        format!("1\n{}\n", m.total_volume)
                    }
                    Some("LIST") => {
                        if let Some(_) = parts.next() {
                            "1\nERROR: usage LIST\n".to_string()
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
                    // TODO: maybe it is valid to put this inside the separate thread? 
                    Some("PING_TCP") => {
                        use std::collections::hash_map::DefaultHasher;
                        use std::hash::{Hash, Hasher};

                        let mut hasher = DefaultHasher::new();
                        SystemTime::now().hash(&mut hasher);
                        let hash = hasher.finish();

                        // 200 - 1200 ms of timeoout
                        let fake_random_timeout = 200 + (hash % 10000 / 10);

                        thread::sleep(Duration::from_millis(fake_random_timeout));
                        // there is no need to add the header here as the user does not see this
                        // output
                        "PONG\n".to_string()
                    }
                    Some("OPEN_UDP_CONNECTIONS") => {
                        format!("1\n{}", udp_stop_flags.iter().filter(|stop_flag| !stop_flag.load(Ordering::Relaxed)).count())
                    }
                    Some("ALL_UDP_CONNECTIONS") => format!("1\n{}", udp_stop_flags.len()),
                    // In out client we are sending the bye message, ni need to add anything in
                    // here. Although this may create complications when we are testing with nc
                    Some("EXIT") => break,
                    Some("HELP") => {
                            "12\nAvailable commands are:
                                1. STREAM 127.0.0.1:<port_number> <ticker-1>,<ticker-2>,...,<ticker-n> - Create a 
                                    broadcast to the provided address where you will recieve live financial data 
                                    updates every second, excluding the first one.
                                2. GET <ticker> - Get latest financial information for the given ticker.
                                3. GET_MANY <ticker-1>,<ticker-2>,...,<ticker-n> - Get the latest financial information for the given tickers.
                                4. GET_TOTAL_VOLUME - sekf explanatory namimg.
                                5. LIST - Dump all of the latest financial information.
                                6. PING_TCP - TODO:
                                7. OPEN_UDP_CONNECTIONS - Get the number of open UDP connections in this session.
                                8. ALL_UDP_CONNECTIONS - Get the number of the total UDP connections in this session.
                                9. EXIT
                                \n".to_string()
                    }
                    _ => "1\nERROR: Unknown command\n".to_string(),
                };

                let _ = writer.write_all(response.as_bytes());
                let _ = writer.flush();
            }
            Err(_) => {
                break;
            }
        }
    }

    for stop_flag in udp_stop_flags {
        stop_flag.store(true, Ordering::Relaxed);
    }
}
