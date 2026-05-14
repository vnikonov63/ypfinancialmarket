use std::{
    net::TcpListener,
    sync::{Arc, Mutex},
    thread,
};

use crate::{server::handle_client, stock::StockMarket};

mod server;
mod stock;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    println!("Server is listening on port 7878");

    let stock_market = Arc::new(Mutex::new(StockMarket::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Detected a new connection");
                let stock_market = Arc::clone(&stock_market);
                thread::spawn(move || {
                    handle_client(stream, stock_market);
                });
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }

    Ok(())
}
