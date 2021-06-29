mod config;
extern crate reqwest;
use core::f64;

use chrono::prelude::*;
use error_chain::error_chain;
use reqwest::Client;
use serde::Deserialize;
use std::process;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct MarketData {
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
struct MarketResponse {
    success: bool,
    data: Vec<MarketData>,
}
#[derive(Deserialize, Debug)]
struct GetMarketsResponse {
    success: bool,
    data: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let c = config::Config::new().unwrap_or_else(|err| {
        panic!("Config Err: {:?}", err);
        // process::exit(1);
    });
    // print_markets(&c.client).await?;

    let s = c.symbol.replace(&['/'][..], "");
    let q = format!("https://serum-api.bonfida.com/trades/{}", s);

    // Perform the actual execution of the network request
    let res = c.client.get(q).send().await?;

    // Parse the response body as Json in this case
    let res = match res.json::<MarketResponse>().await {
        Ok(r) => r,
        Err(e) => {
            if e.to_string().contains("Market does not exist") {
                println!("Market not found for {}", c.symbol);
                print_markets(&c.client).await?;
            } else {
                println!("Decode Err: {}", e);
            }
            process::exit(1);
        }
    };
    let d = res.data;

    let close = d.iter().next().unwrap();
    let close_time = close.time;
    let close_price = close.price;

    // store the indexes
    let mut open: usize = 0;
    let mut high: usize = 0;
    let mut low: usize = 0;

    let interval_ms = c.interval.num_milliseconds() as f64;
    for (i, o) in d.iter().enumerate() {
        // println!("{} - {}", o.time, o.price);
        if (close_time - o.time) > interval_ms {
            open = i - 1;
            println!("Breaking out of loop");
            break;
        }
        if low == 0 || o.price < d[low].price {
            low = i;
        }
        if high == 0 || o.price > d[high].price {
            high = i;
        }
    }
    if open == 0 {
        open = d.len() - 1;
    }
    let open_price = d[open].price;
    let open_time = d[open].time;

    let low_price = d[low].price;
    let high_price = d[high].price;

    let twap = (open_price + close_price + low_price + high_price) / 4.0;

    let open_time = NaiveDateTime::from_timestamp((open_time as i64) / 1000, 0);
    let close_time = NaiveDateTime::from_timestamp((close_time as i64) / 1000, 0);

    println!("TWAP Interval: {} minute(s)", c.interval.num_minutes());
    println!("Open: ${:.2} ({})", open_price, open_time);
    println!("High: ${:.2}", high_price);
    println!("Low: ${:.2}", low_price);
    println!("Close: ${:.2} ({})", close_price, close_time);
    println!("");
    println!("TWAP: ${:.2}", twap);

    Ok(())
}

async fn print_markets(cli: &Client) -> Result<()> {
    let q = format!("https://serum-api.bonfida.com/pairs");
    // Perform the actual execution of the network request
    let res = match cli.get(q).send().await {
        Ok(i) => i,
        Err(e) => panic!("Get Markets err: {}", e),
    };

    // Parse the response body as Json in this case
    let res = res.json::<GetMarketsResponse>().await;
    let d = res.unwrap().data;
    // println!("{:?}", d);
    for i in d.iter() {
        println!(" >{}", i);
    }
    Ok(())
}
