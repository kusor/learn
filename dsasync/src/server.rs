use tide::prelude::*;
use tide::{Request, Response};

use crate::actors::data_buffering::DataBufferingActor;
use crate::messages::RequestMsg;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Symbol {
    pub symbol: String,
}

impl Default for Symbol {
    fn default() -> Self {
        Self {
            symbol: String::from(""),
        }
    }
}

#[derive(Clone)]
pub struct State {
    actor: xactor::Addr<DataBufferingActor>,
}

impl State {
    pub fn new(actor: xactor::Addr<DataBufferingActor>) -> Self {
        Self { actor }
    }
}

///
/// Handle the request using HTTP GET method like
/// `curl 'localhost:8080/tail/123?symbol=APPLE'`
///
pub async fn handle_request(req: Request<State>) -> tide::Result<Response> {
    let mut n: usize = req.param("n")?.parse().unwrap_or(0);
    if n > 500 {
        n = 500
    }

    let symbol: Symbol = req.query()?;
    let sym = symbol.symbol;

    let state = req.state();
    let addr = &state.actor;
    let messages = addr.call(RequestMsg { n, sym }).await?;

    Ok(Response::builder(200).body(json!(&messages)).build())
}
