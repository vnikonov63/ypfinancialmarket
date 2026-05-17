use rkyv::rancor::Error;

use std::{
    collections::HashSet,
    net::UdpSocket,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use crate::stock::{StockMarket, StockQuote};

pub struct StockMarketSenderUDP {
    pub socket: UdpSocket,
    tickers: HashSet<String>,
    stock_market: Arc<Mutex<StockMarket>>,
    stop: Arc<AtomicBool>,
}

impl StockMarketSenderUDP {
    pub fn new(
        bind_addr: &str,
        tickers: HashSet<String>,
        stop: Arc<AtomicBool>,
        stock_market: Arc<Mutex<StockMarket>>,
    ) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(bind_addr)?;
        //TODO: read more about the nonblocking mode.
        socket.set_nonblocking(true)?;
        Ok(Self {
            socket,
            tickers,
            stock_market,
            stop,
        })
    }

    fn send_to(&self, target_addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let m = self.stock_market.lock().unwrap();
        let mut selected_stocks: Vec<StockQuote> = Vec::new();

        for ticker in &self.tickers {
            // we are checking the validiy of the ticker at the TCP Server
            // So it is quaranteed to exist.
            if let Some(stock) = m.stocks.get(ticker) {
                selected_stocks.push(stock.clone());
            }
        }

        let encoded = rkyv::to_bytes::<Error>(&selected_stocks)?;
        self.socket.send_to(&encoded, target_addr)?;

        Ok(())
    }

    // Note that here we are monitoring each of the UDP connections, when the user wants stock
    // streaming. Note that the TCP connection is different and has a PING system of its own.
    // For Clarity I made a distinguishement between PING_UDP and PING_TCP.
    pub fn start_broadcasting_with_ping(
        &self,
        target_addr: &str,
        interval_ms: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // We are checking for PINGs every 250 milliseconds.
        // This number is different from the one when we consdier a client
        // non-responsive and close the connection. The later is 5 seconds.
        // Also we need to send the
        let ping_timeout = Duration::from_millis(250);
        // x20 of ping
        let read_grace_timeout = Duration::from_secs(5);
        // x4 of ping
        let send_timeout = Duration::from_millis(interval_ms);

        let mut last_read = Instant::now();
        // This makes it so we skip the first second. The user shouldn't notice it
        // We can so: Instant::now() - send_timeout;
        let mut last_send = Instant::now();
        let mut buf = [0u8; 1024];

        loop {
            // Just to make sure that we do not continue the UDP connection if
            // there is no reason to do so.
            if self.stop.load(Ordering::Relaxed) {
                break;
            }

            match self.socket.recv(&mut buf) {
                Ok(size) => {
                    let msg = String::from_utf8_lossy(&buf[..size]);
                    if msg.trim() == "PING_UDP" {
                        last_read = Instant::now();
                    }
                }
                // Ddo not use `use` at the top file for the error types
                // and implement and use the match guards more.
                // The purpuse of this
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(_) => break,
            }

            // We didn't get a Ping in 5secs, shutting dowm.
            if last_read.elapsed() >= read_grace_timeout {
                self.stop.store(true, Ordering::Relaxed);
                break;
            }

            if last_send.elapsed() > send_timeout {
                match self.send_to(target_addr) {
                    Ok(()) => {
                        //TODO: add the ability to have logs here
                        println!("Send most recent financial data");
                    }
                    Err(e) => {
                        eprintln!("Error while sending the data to the UDP client: {}", e);
                    }
                }
                last_send = Instant::now();
            }

            // choosing the smallest common denominator
            thread::sleep(ping_timeout);
        }

        Ok(())
    }
}
