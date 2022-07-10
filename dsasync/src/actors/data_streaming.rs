use crate::DataStreamingMsg;
use crate::ProcessQuoteMsg;
use chrono::prelude::*;
use std::io::{Error, ErrorKind};
use xactor::*;
use yahoo_finance_api as yahoo;
///
/// Data Streaming Actor
///
#[derive(Default)]
pub struct DataStreamingActor;

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
