use rkyv::{Archive, Deserialize, Serialize};

use std::{
    collections::HashMap,
    fmt,
    fs::File,
    io::{self, BufRead, BufReader},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
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

    pub fn update_data(&mut self) -> u32 {
        // if price grows, so does the volume, people flock.

        let price_change_percent = Self::fake_random_percent(2.0);
        let volume_change_percent = Self::fake_random_percent(5.0);

        let price_goes_up = Self::fake_random_bool();
        let volume_goes_up = Self::fake_random_bool();

        if price_goes_up {
            self.price *= 1.0 + price_change_percent / 100.0;
        } else {
            self.price *= 1.0 - price_change_percent / 100.0;
        }

        let volume_as_f64 = self.volume as f64;

        let new_volume = if volume_goes_up {
            volume_as_f64 * (1.0 + volume_change_percent / 100.0)
        } else {
            volume_as_f64 * (1.0 - volume_change_percent / 100.0)
        };

        self.volume = new_volume.round() as u32;
        self.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        new_volume.round() as u32
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

    fn fake_random_percent(max_percent: f64) -> f64 {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        let value = (nanos % 10_000) as f64 / 10_000.0;

        value * max_percent
    }

    fn fake_random_bool() -> bool {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();

        nanos % 2 == 0
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
    pub fn new(ticker_path: PathBuf) -> std::io::Result<Self> {
        let file = File::open(ticker_path)?;
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

    pub fn update(&mut self) {
        self.total_volume = 0;
        for stock in self.stocks.values_mut() {
            self.total_volume += stock.update_data();
        }
    }
}
