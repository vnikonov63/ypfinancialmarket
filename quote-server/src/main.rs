use clap::Parser;
use log::{debug, error, info, trace};

use std::{
    net::{SocketAddr, TcpListener},
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{server::handle_client, stock::StockMarket};

mod server;
mod stock;
mod udpsender;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short = 'a', long, default_value = "127.0.0.1:7878")]
    addr_tcp: SocketAddr,
    #[arg(short = 'f', long, default_value = "data/tickers.txt")]
    ticker_list_path: PathBuf,
    // I think I should not use this one, because I am creating multiple UDP
    // connections.
    /* #[arg(short = 'p', long, default_value_t = 7879)]
    port_udp: u16, */
}

fn main() -> std::io::Result<()> {
    // To see logs run with RUST_LOG=level_of_choice cargo run
    // The selevs are LOWEST IMPORTANCE trace -> debug -> info -> warn -> error HIGHEST_IMPORTANCE
    tracing_subscriber::fmt()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();

    let args = Args::parse();
    let listener = TcpListener::bind(args.addr_tcp)?;
    info!("Server is listening on port 7878");

    let stock_market = StockMarket::new(args.ticker_list_path)?;
    trace!("{:?}", stock_market);

    let stock_market = Arc::new(Mutex::new(stock_market));
    let stock_market_update_thread = Arc::clone(&stock_market);

    thread::spawn(move || {
        loop {
            {
                let mut market = stock_market_update_thread.lock().unwrap();
                market.update();
                debug!("Stock Market auto update was successfull");
                trace!("{:?}", market);
            }

            thread::sleep(Duration::from_secs(2));
        }
    });

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                info!("Detected a new connection");
                let stock_market = Arc::clone(&stock_market);
                thread::spawn(move || {
                    handle_client(stream, stock_market);
                });
            }
            Err(e) => error!("Connection failed: {}", e),
        }
    }

    Ok(())
}
