use async_std::{fs, prelude::*, stream};
use async_trait::async_trait;
use chrono::prelude::*;
use clap::Parser;
use futures::future::join_all;
use std::io::{Error, ErrorKind};
use std::time;
use xactor::*;
use yahoo_finance_api as yahoo;

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
/// A trait to provide a common interface for all signal calculations.
///
#[async_trait]
trait AsyncStockSignal {
    ///
    /// The signal's data type.
    ///
    type SignalType;

    ///
    /// Calculate the signal on the provided series.
    ///
    /// # Returns
    ///
    /// The signal (using the provided type) or `None` on error/invalid data.
    ///
    async fn calculate(&self, series: &[f64]) -> Option<Self::SignalType>;
}

///
/// Find the maximum in a series of f64
///
struct MaxPrice {}

#[async_trait]
impl AsyncStockSignal for MaxPrice {
    type SignalType = f64;

    async fn calculate(&self, series: &[f64]) -> Option<Self::SignalType> {
        if series.is_empty() {
            None
        } else {
            Some(series.iter().fold(f64::MIN, |acc, q| acc.max(*q)))
        }
    }
}

///
/// Calculates the absolute and relative difference between the beginning and ending of an f64 series. The relative difference is relative to the beginning.
///
struct PriceDifference {}

#[async_trait]
impl AsyncStockSignal for PriceDifference {
    type SignalType = (f64, f64);

    /// # Returns
    ///
    /// A tuple `(absolute, relative)` difference.
    ///
    async fn calculate(&self, a: &[f64]) -> Option<(f64, f64)> {
        if !a.is_empty() {
            // unwrap is safe here even if first == last
            let (first, last) = (a.first().unwrap(), a.last().unwrap());
            let abs_diff = last - first;
            let first = if *first == 0.0 { 1.0 } else { *first };
            let rel_diff = abs_diff / first;
            Some((abs_diff, rel_diff))
        } else {
            None
        }
    }
}

///
/// Window function to create a simple moving average
///
struct WindowedSMA {
    window_size: usize,
}

#[async_trait]
impl AsyncStockSignal for WindowedSMA {
    type SignalType = Vec<f64>;

    async fn calculate(&self, series: &[f64]) -> Option<Vec<f64>> {
        if !series.is_empty() && self.window_size > 1 {
            Some(
                series
                    .windows(self.window_size)
                    .map(|w| w.iter().sum::<f64>() / w.len() as f64)
                    .collect(),
            )
        } else {
            None
        }
    }
}

///
/// Find the minimum in a series of f64
///
struct MinPrice {}

#[async_trait]
impl AsyncStockSignal for MinPrice {
    type SignalType = f64;

    async fn calculate(&self, series: &[f64]) -> Option<f64> {
        if series.is_empty() {
            None
        } else {
            Some(series.iter().fold(f64::MAX, |acc, q| acc.min(*q)))
        }
    }
}

///
/// xactor message
///
#[message]
struct DataStreamingMsg {
    symbols: String,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
}

///
/// Data Streaming do everything actor
///
struct DataStreamingActor;

impl Actor for DataStreamingActor {}

#[async_trait::async_trait]
impl Handler<DataStreamingMsg> for DataStreamingActor {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: DataStreamingMsg) {
        // a simple way to output a CSV header
        println!("period start,symbol,price,change %,min,max,30d avg");
        let symbols: Vec<&str> = msg.symbols.split(',').collect();
        let fut: Vec<_> = symbols
            .iter()
            .map(|symbol| get_symbol_values(symbol, &msg.from, &msg.to))
            .collect();
        let _ = join_all(fut).await;
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

///
/// Do all the calculations for a given symbol once the data is fetch
///
async fn get_symbol_values(
    symbol: &str,
    from: &DateTime<Utc>,
    to: &DateTime<Utc>,
) -> Option<Vec<f64>> {
    let closes = fetch_closing_data(symbol, from, to).await.ok()?;
    if !closes.is_empty() {
        // min/max of the period. unwrap() because those are Option types
        let max_price = MaxPrice {};
        let period_max: f64 = max_price.calculate(&closes).await?;
        let min_price = MinPrice {};
        let period_min: f64 = min_price.calculate(&closes).await?;
        let last_price = *closes.last().unwrap_or(&0.0);
        let price_diff = PriceDifference {};
        let (_, pct_change) = price_diff.calculate(&closes).await?;
        let w_sma = WindowedSMA { window_size: 30 };
        let sma = w_sma.calculate(&closes).await?;

        // a simple way to output CSV data
        println!(
            "{},{},${:.2},{:.2}%,${:.2},${:.2},${:.2}",
            from.to_rfc3339(),
            symbol,
            last_price,
            pct_change * 100.0,
            period_min,
            period_max,
            sma.last().unwrap_or(&0.0)
        );
    }
    Some(closes)
}

#[xactor::main]
async fn main() -> Result<()> {
    let opts = Opts::parse();
    let from: DateTime<Utc> = opts.from.parse().expect("Couldn't parse 'from' date");
    let to = Utc::now();
    let fcontents = fs::read_to_string("sp500.txt").await?;

    let mut interval = stream::interval(time::Duration::from_secs(30));
    // Start actor and get its address
    let addr = DataStreamingActor.start().await?;

    // NOTE: The Stream::interval is still unstable
    while interval.next().await.is_some() {
        addr.call(DataStreamingMsg {
            symbols: fcontents.clone(),
            from,
            to,
        })
        .await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use super::*;

    #[async_std::test]
    async fn test_PriceDifference_calculate() -> std::io::Result<()> {
        let signal = PriceDifference {};
        assert_eq!(signal.calculate(&[]).await, None);
        assert_eq!(signal.calculate(&[1.0]).await, Some((0.0, 0.0)));
        assert_eq!(signal.calculate(&[1.0, 0.0]).await, Some((-1.0, -1.0)));
        assert_eq!(
            signal
                .calculate(&[2.0, 3.0, 5.0, 6.0, 1.0, 2.0, 10.0])
                .await,
            Some((8.0, 4.0))
        );
        assert_eq!(
            signal.calculate(&[0.0, 3.0, 5.0, 6.0, 1.0, 2.0, 1.0]).await,
            Some((1.0, 1.0))
        );
        Ok(())
    }

    #[async_std::test]
    async fn test_MinPrice_calculate() -> std::io::Result<()> {
        let signal = MinPrice {};
        assert_eq!(signal.calculate(&[]).await, None);
        assert_eq!(signal.calculate(&[1.0]).await, Some(1.0));
        assert_eq!(signal.calculate(&[1.0, 0.0]).await, Some(0.0));
        assert_eq!(
            signal
                .calculate(&[2.0, 3.0, 5.0, 6.0, 1.0, 2.0, 10.0])
                .await,
            Some(1.0)
        );
        assert_eq!(
            signal.calculate(&[0.0, 3.0, 5.0, 6.0, 1.0, 2.0, 1.0]).await,
            Some(0.0)
        );
        Ok(())
    }

    #[async_std::test]
    async fn test_MaxPrice_calculate() -> std::io::Result<()> {
        let signal = MaxPrice {};
        assert_eq!(signal.calculate(&[]).await, None);
        assert_eq!(signal.calculate(&[1.0]).await, Some(1.0));
        assert_eq!(signal.calculate(&[1.0, 0.0]).await, Some(1.0));
        assert_eq!(
            signal
                .calculate(&[2.0, 3.0, 5.0, 6.0, 1.0, 2.0, 10.0])
                .await,
            Some(10.0)
        );
        assert_eq!(
            signal.calculate(&[0.0, 3.0, 5.0, 6.0, 1.0, 2.0, 1.0]).await,
            Some(6.0)
        );
        Ok(())
    }

    #[async_std::test]
    async fn test_WindowedSMA_calculate() -> std::io::Result<()> {
        let series = vec![2.0, 4.5, 5.3, 6.5, 4.7];

        let signal = WindowedSMA { window_size: 3 };
        assert_eq!(
            signal.calculate(&series).await,
            Some(vec![3.9333333333333336, 5.433333333333334, 5.5])
        );

        let signal = WindowedSMA { window_size: 5 };
        assert_eq!(signal.calculate(&series).await, Some(vec![4.6]));

        let signal = WindowedSMA { window_size: 10 };
        assert_eq!(signal.calculate(&series).await, Some(vec![]));
        Ok(())
    }
}
