use crate::PrintableMsg;
use async_std::fs;
use async_std::io::WriteExt;
use xactor::*;

///
/// Data Writing Actor
///
pub struct DataWritingActor {
    pub file: Option<fs::File>,
    pub filename: String,
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
