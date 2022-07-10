use crate::{
    AsyncStockSignal, Handler, MaxPrice, MinPrice, PriceDifference, PrintableMsg, ProcessQuoteMsg,
    WindowedSMA,
};
use xactor::*;

///
/// Data Processing Actor
///
#[derive(Default)]
pub struct DataProcessingActor;

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
