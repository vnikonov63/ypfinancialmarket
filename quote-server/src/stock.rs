use std::{
    collections::HashMap,
    fmt,
    fs::File,
    io::{self, BufRead, BufReader},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone)]
pub struct StockQuote {
    pub ticker: String,
    pub price: f64,
    pub volume: u32,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct StockMarket {
    pub stocks: HashMap<String, StockQuote>,
    pub total_volume: u32,
}

impl fmt::Display for StockQuote {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}",
            self.ticker, self.price, self.volume, self.timestamp
        )
    }
}

impl StockQuote {
    pub fn new(ticker: &str, price: f64, volume: u32) -> Self {
        Self {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            price,
            volume,
            ticker: String::from(ticker),
        }
    }

    pub fn formatted_time(&self) -> String {
        format!("{}s", self.timestamp)
    }

    // make sure that this has a really random thing and the fake random one
    // using the Rust feature technique.
    pub fn random() -> Self {
        todo!()
    }

    pub fn fake_random_for_ticker(ticker: &str) -> Self {
        Self {
            ticker: ticker.to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            price: Self::fake_random_price(),
            volume: Self::fake_random_volume(),
        }
    }
    fn fake_random_price() -> f64 {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();

        50.0 + (nanos % 25_000) as f64 / 100.0
    }

    fn fake_random_volume() -> u32 {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();

        100 + nanos % 25_000
    }

    pub fn from_string(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('|').collect();
        if parts.len() == 4 {
            Some(StockQuote {
                ticker: parts[0].to_string(),
                price: parts[1].parse::<f64>().ok()?,
                volume: parts[2].parse::<u32>().ok()?,
                timestamp: parts[3].parse::<u64>().ok()?,
            })
        } else {
            None
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.ticker.as_bytes());
        bytes.push(b'|');
        bytes.extend_from_slice(self.price.to_string().as_bytes());
        bytes.push(b'|');
        bytes.extend_from_slice(self.volume.to_string().as_bytes());
        bytes.push(b'|');
        bytes.extend_from_slice(self.timestamp.to_string().as_bytes());

        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let s = std::str::from_utf8(bytes).ok()?;
        Self::from_string(s)
    }
}

impl StockMarket {
    pub fn new() -> std::io::Result<Self> {
        let file = File::open("data/tickers.txt")?;
        let reader = BufReader::new(file);

        let mut stocks = HashMap::new();
        let mut total_volume: u32 = 0;
        for line in reader.lines() {
            let ticker = line?;
            let stock = StockQuote::fake_random_for_ticker(ticker.as_str());
            total_volume += stock.volume;
            stocks.insert(ticker, stock);
        }
        Ok(Self {
            stocks,
            total_volume,
        })
    }
}
