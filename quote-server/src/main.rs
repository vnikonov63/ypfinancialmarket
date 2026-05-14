use std::{
    net::TcpListener,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{server::handle_client, stock::StockMarket};

mod server;
mod stock;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    println!("Server is listening on port 7878");

    let stock_market = StockMarket::new()?;
    println!("{:?}", stock_market);

    let stock_market = Arc::new(Mutex::new(stock_market));
    let stock_market_update_thread = Arc::clone(&stock_market);

    thread::spawn(move || {
        loop {
            {
                let mut market = stock_market_update_thread.lock().unwrap();
                market.update();
                println!("{:?}", market);
            }

            thread::sleep(Duration::from_secs(2));
        }
    });

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
