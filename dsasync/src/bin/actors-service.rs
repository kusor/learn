use async_std::{fs, prelude::*, stream};
use chrono::prelude::*;
use clap::Parser;
use std::time;
use xactor::*;

use crate::actors::data_buffering::DataBufferingActor;
use crate::actors::data_printing::DataPrintingActor;
use crate::actors::data_procesing::DataProcessingActor;
use crate::actors::data_streaming::DataStreamingActor;
use crate::actors::data_writing::DataWritingActor;
use crate::async_stock_signal::*;
use crate::messages::*;

#[path = "../actors/mod.rs"]
mod actors;
#[path = "../async-stock-signal.rs"]
mod async_stock_signal;
#[path = "../messages.rs"]
mod messages;

#[derive(Parser, Debug)]
#[clap(
    version = "1.0",
    author = "Claus Matzinger",
    about = "A Manning LiveProject: async Rust"
)]
struct Opts {
    #[clap(short, long, default_value = "AAPL,MSFT,UBER,GOOG")]
    symbols: String,
    #[clap(short, long)]
    from: String,
}

#[xactor::main]
async fn main() -> Result<()> {
    let opts = Opts::parse();
    let from: DateTime<Utc> = opts.from.parse().expect("Couldn't parse 'from' date");
    let to = Utc::now();
    let fcontents = fs::read_to_string("sp500.txt").await?;
    let symbols: Vec<&str> = fcontents.split(',').collect();

    let mut interval = stream::interval(time::Duration::from_secs(5));
    // Start actor
    let _dsactor = DataStreamingActor::start_default().await?;
    let _dprocactor = DataProcessingActor::start_default().await?;
    let _dprintactor = DataPrintingActor::start_default().await?;
    let _dwractor = DataWritingActor {
        filename: format!("{}.csv", to.timestamp()),
        file: None,
    }
    .start()
    .await?;
    let _dbuffactor = DataBufferingActor {
        n: 500,
        queue: None,
    }
    .start()
    .await?;

    // a simple way to output a CSV header
    println!("period start,symbol,price,change %,min,max,30d avg");
    // NOTE: The Stream::interval is still unstable
    while interval.next().await.is_some() {
        for symbol in &symbols {
            let _ = Broker::from_registry().await?.publish(DataStreamingMsg {
                symbol: symbol.to_string(),
                from,
                to,
            });
        }
    }
    Ok(())
}
