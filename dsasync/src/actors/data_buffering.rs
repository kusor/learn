use crate::PrintableMsg;
use std::collections::VecDeque;
use xactor::*;

///
/// Data Buffering Actor
///
pub struct DataBufferingActor {
    pub queue: Option<VecDeque<PrintableMsg>>,
    pub n: usize,
}

#[async_trait::async_trait]
impl Actor for DataBufferingActor {
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        self.queue = Some(VecDeque::with_capacity(self.n));

        println!("DataBufferingActor subscribed to PrintableMsg");
        let _ = ctx.subscribe::<PrintableMsg>().await;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Handler<PrintableMsg> for DataBufferingActor {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: PrintableMsg) {
        if let Some(queue) = &mut self.queue {
            if queue.len() > 0 && queue.len() >= self.n {
                queue.pop_back();
            }
            queue.push_front(msg);
        }
    }
}
