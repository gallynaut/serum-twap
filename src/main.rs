mod config;
extern crate reqwest;
use core::f64;
use reqwest::Client;
use serde::Deserialize;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct OHLC {
    open: Option<f64>,
    high: Option<f64>,
    low: Option<f64>,
    close: Option<f64>,
    volume: Option<f64>,
}
impl OHLC {
    fn new() -> Self {
        Self {
            open: None,
            high: None,
            low: None,
            close: None,
            volume: None,
        }
    }
    fn print(&self) {
        if !self.is_valid() {
            return;
        }
        println!(
            "O: {:.2}, H: {:.2}, L: {:.2}, C: {:.2}",
            self.open.unwrap(),
            self.high.unwrap(),
            self.low.unwrap(),
            self.close.unwrap()
        );
    }
    fn is_valid(&self) -> bool {
        if self.open == None || self.high == None || self.low == None || self.close == None {
            return false;
        }
        true
    }
    fn interpolate(&self, other: OHLC) -> OHLC {
        if !self.is_valid() || !other.is_valid() {
            return OHLC::new();
        }
        OHLC {
            open: Some((self.open.unwrap() + other.open.unwrap()) / 2.0),
            high: Some((self.high.unwrap() + other.high.unwrap()) / 2.0),
            low: Some((self.low.unwrap() + other.low.unwrap()) / 2.0),
            close: Some((self.close.unwrap() + other.close.unwrap()) / 2.0),
            volume: None,
        }
    }
    fn twap(&self) -> Option<f64> {
        if !self.is_valid() {
            return None;
        }
        let twap =
            (self.open.unwrap() + self.high.unwrap() + self.low.unwrap() + self.close.unwrap())
                / 4.0;
        Some(twap)
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MarketData {
    market: String,
    price: f64,
    size: f64,
    side: String,
    time: f64,
    order_id: String,
    fee_cost: f64,
    market_address: String,
}
#[derive(Deserialize, Debug)]
pub struct MarketResponse {
    pub success: bool,
    pub data: Vec<MarketData>,
    candles: Option<[OHLC; 24]>,
}
impl MarketResponse {
    fn is_valid(&self) -> bool {
        self.success
    }
    pub fn get_hourly_candles(&self) -> Option<[OHLC; 24]> {
        if !self.is_valid() {
            return None;
        }
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let start_us: f64 = since_the_epoch.as_millis() as f64;
        let interval_us = 3600000.0; //

        // sort responses into hourly array
        let mut candle_data: [Vec<MarketData>; 24] = Default::default();
        for o in self.data.iter() {
            let i = (start_us - o.time) / interval_us;
            let i = i as usize;
            candle_data[i].push(o.clone());
        }

        // process each hour of data and compute OHLC
        let mut candles = [OHLC::new(); 24];
        for (i, x) in candle_data.iter().enumerate() {
            if x.len() == 0 {
                continue;
            }
            // reverse order - does Vec preserve order?
            candles[i].open = match x.iter().last() {
                Some(i) => Some(i.price),
                None => None,
            };
            candles[i].close = match x.iter().next() {
                Some(i) => Some(i.price),
                None => None,
            };

            let mut high: Option<f64> = None;
            let mut low: Option<f64> = None;

            for y in x.iter() {
                if high == None || y.price > high.unwrap() {
                    high = Some(y.price)
                }
                if low == None || y.price < low.unwrap() {
                    low = Some(y.price)
                }
            }
            candles[i].low = low;
            candles[i].high = high;
        }
        Some(candles)
    }
}
#[derive(Deserialize, Debug)]
struct GetMarketsResponse {
    success: bool,
    data: Vec<String>,
}
impl GetMarketsResponse {
    fn print_markets(&self) {
        for i in self.data.iter() {
            println!(" > {}", i);
        }
    }
}

#[tokio::main]
async fn main() {
    let c = config::Config::new().unwrap_or_else(|err| {
        panic!("Config Err: {:?}", err);
        // process::exit(1);
    });

    let q = format!("https://serum-api.bonfida.com/trades/{}", c.symbol);

    // Perform the network request
    let res = match c.client.get(q).send().await {
        Ok(r) => r,
        Err(e) => panic!("Req Err: {}", e),
    };

    // Parse the response body as Json
    let res = match res.json::<MarketResponse>().await {
        Ok(r) => r,
        Err(e) => {
            if e.to_string().contains("Market does not exist") {
                println!("Market not found for {}", c.symbol);
                let _ = get_markets(&c.client).await;
            } else {
                println!("Decode Err: {}", e);
            }
            process::exit(1);
        }
    };
    println!("");
    println!("Total Trades: {}", res.data.len());

    let candles = res.get_hourly_candles().unwrap();

    let new_candles = match smooth_candles(&candles) {
        Some(i) => i,
        None => candles,
    };

    let twap = match calculate_twap(&new_candles) {
        Some(i) => i,
        None => panic!("error calculating TWAP"),
    };
    println!("");
    println!("TWAP: ${:.2}", twap);
}

// get list of markets from API call
async fn get_markets(cli: &Client) -> Result<(), &'static str> {
    let q = format!("https://serum-api.bonfida.com/pairs");
    // Perform the actual execution of the network request
    let res = match cli.get(q).send().await {
        Ok(i) => i,
        Err(e) => panic!("Get Markets err: {}", e),
    };

    // Parse the response body as Json in this case
    let res = res.json::<GetMarketsResponse>().await;
    let _ = res.unwrap().print_markets();
    Ok(())
}

// if a candle is missing data, interpolate the candle using prev/next candle
fn smooth_candles(candles: &[OHLC; 24]) -> Option<[OHLC; 24]> {
    let mut new_candles = [OHLC::new(); 24];
    for (i, c) in candles.iter().enumerate() {
        if c.is_valid() {
            new_candles[i] = c.clone();
            continue;
        }
        let mut next_candle: Option<OHLC> = None;
        if i != candles.len() {
            next_candle = Some(candles[i + 1].clone())
        }
        let mut prev_candle: Option<OHLC> = None;
        if i != 0 {
            prev_candle = Some(candles[i - 1].clone())
        }
        if !next_candle.is_none() {
            if !prev_candle.is_none() {
                new_candles[i] = next_candle.unwrap().interpolate(prev_candle.unwrap())
            } else {
                new_candles[i] = next_candle.unwrap()
            }
        } else {
            if !prev_candle.is_none() {
                new_candles[i] = prev_candle.unwrap()
            } else {
                println!("Cant smooth candle for {}", i);
            }
        }
    }
    Some(new_candles)
}

// calculate the twap using the candles
fn calculate_twap(candles: &[OHLC; 24]) -> Option<f64> {
    let mut running_total = 0.0;
    let mut invalid: usize = 0;
    for c in candles.iter() {
        match c.twap() {
            Some(i) => running_total += i,
            None => invalid += 1,
        }
    }
    let twap = running_total / ((24 - invalid) as f64);
    Some(twap)
}
