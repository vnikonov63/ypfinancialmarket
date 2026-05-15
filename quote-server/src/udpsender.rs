use rkyv::rancor::Error;

use std::{
    collections::HashSet,
    net::UdpSocket,
    sync::{Arc, Mutex, atomic::AtomicBool},
    thread,
    time::Duration,
};

use crate::stock::{StockMarket, StockQuote};

pub struct StockMarketSenderUDP {
    socket: UdpSocket,
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
            if let Some(stock) = m.stocks.get(ticker) {
                selected_stocks.push(stock.clone());
            }
        }

        let encoded = rkyv::to_bytes::<Error>(&selected_stocks)?;
        self.socket.send_to(&encoded, target_addr)?;

        Ok(())
    }

    pub fn start_broadcasting(
        &self,
        target_addr: &str,
        interval_ms: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: We should utilize the AtmoicBool here, so we close the UDP connection once the
        // corresponding TCP Client disconnects.

        loop {
            match self.send_to(target_addr) {
                Ok(()) => {
                    //TODO: add the ability to have logs here
                    println!("Send most recent financial data");
                }
                Err(e) => {
                    eprintln!("Error while sending the data to the UDP client: {}", e);
                }
            }

            thread::sleep(Duration::from_millis(interval_ms));
        }
    }
}
