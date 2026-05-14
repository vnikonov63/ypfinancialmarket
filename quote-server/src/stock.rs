use std::collections::HashMap;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct StockQuote {
    pub ticker: String,
    pub price: f64,
    pub volume: u32,
    pub timestamp: u64,
}

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
    pub fn random() -> Self {
        todo!()
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
