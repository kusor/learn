use async_std::{fs, prelude::*, stream};
use chrono::prelude::*;
use clap::Parser;
use std::io::{Error, ErrorKind};
use std::time;
use xactor::*;
use yahoo_finance_api as yahoo;

use crate::async_stock_signal::*;

#[path = "../async-stock-signal.rs"]
mod async_stock_signal;

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

///
/// xactor message
///
#[message]
#[derive(Clone)]
struct DataStreamingMsg {
    symbol: String,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
}

#[message]
#[derive(Clone)]
struct ProcessQuoteMsg {
    symbol: String,
    closes: Vec<f64>,
    from: DateTime<Utc>,
}

///
/// Data Streaming Actor
///
#[derive(Default)]
struct DataStreamingActor;

#[async_trait::async_trait]
impl Actor for DataStreamingActor {
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        println!("DataStreamingActor subscribed to DataStreamingMsg");
        let _ = ctx.subscribe::<DataStreamingMsg>().await;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Handler<DataStreamingMsg> for DataStreamingActor {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: DataStreamingMsg) {
        match fetch_closing_data(&msg.symbol, &msg.from, &msg.to).await {
            Ok(closes) => match Broker::from_registry().await {
                Ok(mut broker) => {
                    let _ = &broker.publish(ProcessQuoteMsg {
                        symbol: msg.symbol.clone(),
                        closes,
                        from: msg.from,
                    });
                }
                Err(berr) => {
                    eprintln!("Error creating broker: {}", berr);
                }
            },
            Err(e) => {
                eprintln!("Error fetching data for {}: {}", msg.symbol.clone(), e);
            }
        };
    }
}

///
/// Data Processing Actor
///
#[derive(Default)]
struct DataProcessingActor;

#[async_trait::async_trait]
impl Actor for DataProcessingActor {
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        println!("DataProcessingActor subscribed to ProcessQuoteMsg");
        let _ = ctx.subscribe::<ProcessQuoteMsg>().await;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Handler<ProcessQuoteMsg> for DataProcessingActor {
    ///
    /// Do all the calculations for a given symbol once the data is fetch
    ///
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: ProcessQuoteMsg) {
        let closes = &msg.closes;
        // min/max of the period. unwrap() because those are Option types
        let max_price = MaxPrice {};
        let period_max: f64 = max_price.calculate(closes).await.unwrap_or(0.0);
        let min_price = MinPrice {};
        let period_min: f64 = min_price.calculate(closes).await.unwrap_or(0.0);
        let last_price = *closes.last().unwrap_or(&0.0);
        let price_diff = PriceDifference {};
        let (_, pct_change) = price_diff.calculate(closes).await.unwrap_or((0.0, 0.0));
        let w_sma = WindowedSMA { window_size: 30 };
        let sma = w_sma.calculate(closes).await.unwrap_or_default();

        match Broker::from_registry().await {
            Ok(mut broker) => {
                let _ = &broker.publish(PrintableMsg {
                    symbol: msg.symbol.clone(),
                    from: msg.from,
                    last_price,
                    pct_change: pct_change * 100.0,
                    period_min,
                    period_max,
                    sma: *sma.last().unwrap_or(&0.0),
                });
            }
            Err(berr) => {
                eprintln!("Error creating broker: {}", berr);
            }
        }
    }
}

#[message]
#[derive(Clone)]
struct PrintableMsg {
    symbol: String,
    from: DateTime<Utc>,
    last_price: f64,
    pct_change: f64,
    period_max: f64,
    period_min: f64,
    sma: f64,
}

///
/// Data Printing Actor
///
#[derive(Default)]
struct DataPrintingActor;

#[async_trait::async_trait]
impl Actor for DataPrintingActor {
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        println!("DataStreamingActor subscribed to PrintableMsg");
        let _ = ctx.subscribe::<PrintableMsg>().await;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Handler<PrintableMsg> for DataPrintingActor {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: PrintableMsg) {
        // a simple way to output CSV data
        println!(
            "{},{},${:.2},{:.2}%,${:.2},${:.2},${:.2}",
            msg.from.to_rfc3339(),
            msg.symbol,
            msg.last_price,
            msg.pct_change,
            msg.period_min,
            msg.period_max,
            msg.sma
        );
    }
}

///
/// Data Writing Actor
///
struct DataWritingActor {
    file: Option<fs::File>,
    filename: String,
}

#[async_trait::async_trait]
impl Actor for DataWritingActor {
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        let mut file = fs::File::create(&self.filename).await?;

        let _ = writeln!(file, "period start,symbol,price,change %,min,max,30d avg");

        self.file = Some(file);
        println!("DataWritingActor subscribed to PrintableMsg");
        let _ = ctx.subscribe::<PrintableMsg>().await;
        Ok(())
    }

    async fn stopped(&mut self, ctx: &mut Context<Self>) {
        if let Some(file) = &self.file {
            if let Err(e) = file.sync_all().await {
                eprintln!(
                    "Error waiting for data sync for file {}: {}",
                    self.filename.clone(),
                    e
                )
            }
        }
        ctx.stop(None);
        println!("DataWritingActor stopped");
    }
}

#[async_trait::async_trait]
impl Handler<PrintableMsg> for DataWritingActor {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: PrintableMsg) {
        if let Some(file) = &mut self.file {
            let _ = writeln!(
                file,
                "{},{},${:.2},{:.2}%,${:.2},${:.2},${:.2}",
                msg.from.to_rfc3339(),
                msg.symbol,
                msg.last_price,
                msg.pct_change,
                msg.period_min,
                msg.period_max,
                msg.sma
            );
        }
    }
}

///
/// Retrieve data from a data source and extract the closing prices. Errors during download are mapped onto io::Errors as InvalidData.
///
async fn fetch_closing_data(
    symbol: &str,
    beginning: &DateTime<Utc>,
    end: &DateTime<Utc>,
) -> std::io::Result<Vec<f64>> {
    let provider = yahoo::YahooConnector::new();

    let response = provider
        .get_quote_history(symbol, *beginning, *end)
        .await
        .map_err(|_| Error::from(ErrorKind::InvalidData))?;

    let mut quotes = response
        .quotes()
        .map_err(|_| Error::from(ErrorKind::InvalidData))?;

    if !quotes.is_empty() {
        quotes.sort_by_cached_key(|k| k.timestamp);
        Ok(quotes.iter().map(|q| q.adjclose as f64).collect())
    } else {
        Ok(vec![])
    }
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
