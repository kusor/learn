use crate::{PrintableMsg, RequestMsg};
use std::collections::VecDeque;
use xactor::*;

///
/// Data Buffering Actor
///
pub struct DataBufferingActor {
    pub queue: VecDeque<PrintableMsg>,
    pub n: usize,
}

#[async_trait::async_trait]
impl Actor for DataBufferingActor {
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        self.queue = VecDeque::with_capacity(self.n);

        println!("DataBufferingActor subscribed to PrintableMsg");
        let _ = ctx.subscribe::<PrintableMsg>().await;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Handler<PrintableMsg> for DataBufferingActor {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: PrintableMsg) {
        if !self.queue.is_empty() && self.queue.len() >= self.n {
            self.queue.pop_back();
        }
        self.queue.push_front(msg);
    }
}

#[async_trait::async_trait]
impl Handler<RequestMsg> for DataBufferingActor {
    async fn handle(
        &mut self,
        _ctx: &mut Context<Self>,
        msg: RequestMsg,
    ) -> VecDeque<PrintableMsg> {
        // return the messages in the queue up to the given number (n) and/or symbol (sym)
        if !msg.sym.is_empty() {
            let mut q = self.queue.clone();
            q.retain(|x| x.symbol == msg.sym);
            q
        } else {
            self.queue.clone()
        }
    }
}
