use crate::PrintableMsg;
use xactor::*;
///
/// Data Printing Actor
///
#[derive(Default)]
pub struct DataPrintingActor;

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
