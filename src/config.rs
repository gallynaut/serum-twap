use chrono::Duration;
use clap::{App, Arg};
use reqwest::Client;

pub struct Config {
    pub symbol: String,
    pub interval: Duration,
    pub debug: bool,
    pub client: Client,
}

impl Config {
    pub fn new() -> Result<Config, &'static str> {
        // validate command line arguements
        let matches = App::new("Serum-TWAP")
            .version("0.1.0")
            .author("Conner <ConnerNGallagher@gmail.com>")
            .about("using serum to calculate twap")
            .arg(
                Arg::with_name("symbol")
                    .help("the symbol to calculate the TWAP for (BTC/USD)")
                    .index(1)
                    .required(true),
            )
            .arg(
                Arg::with_name("debug")
                    .help("print debug information verbosely")
                    .short("d"),
            )
            .arg(
                Arg::with_name("interval")
                    .short("i")
                    .help("the interval to calculate the TWAP over in minutes")
                    .takes_value(true)
                    .default_value("1440")
                    .required(false),
            )
            .get_matches();

        let symbol = matches
            .value_of("symbol")
            .unwrap()
            .to_string()
            .to_ascii_uppercase()
            .replace(&['/'][..], ""); // remove backslash if provided
        println!("{:.<20} {}", "symbol", symbol);

        let interval = matches
            .value_of("interval")
            .unwrap()
            .parse::<i64>()
            .unwrap();
        if interval == 0 || interval > 1440 {
            // panic
            return Err("interval should be between 1 and 1440 minutes (1 day)");
        }
        let interval = Duration::seconds(interval.checked_mul(60).unwrap());
        println!(
            "{:.<20} {} minute(s)",
            "TWAP interval",
            interval.num_minutes()
        );
        let debug = matches.is_present("debug");

        // Build the client using the builder pattern
        let client = match reqwest::Client::builder().build() {
            Ok(c) => c,
            Err(e) => panic!("Client Err: {}", e),
        };
        Ok(Config {
            symbol,
            interval,
            debug,
            client,
        })
    }
}
