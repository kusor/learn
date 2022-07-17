use tide::prelude::*;
use tide::Request;

#[derive(Debug, Deserialize)]
#[serde(default)]
struct Symbol {
    symbol: String,
}

impl Default for Symbol {
    fn default() -> Self {
        Self {
            symbol: String::from(""),
        }
    }
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    tide::log::start();
    // Create a new tide server
    let mut server = tide::new();
    // Add a route to the given path with param `:n` and method GET
    // to be handled by the provided tide::Endpoint
    server.at("/tail/:n").get(handle_request);
    server.at("/tail/:n").post(handle_post_request);
    server.at("/favicon.ico").get(&favicon);
    // Asynchronously serve the app
    server.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn favicon<R>(_req: R) -> tide::Result {
    Ok(tide::Response::new(200))
}

///
/// Handle the request using HTTP GET method like
/// `curl 'localhost:8080/tail/123?symbol=APPLE'`
///
async fn handle_request(req: Request<()>) -> tide::Result<String> {
    let n: usize = req.param("n")?.parse().unwrap_or(0);
    let mut msg: String = format!("Requested a total of {} records", n);
    let sym: Symbol = req.query()?;
    if !sym.symbol.is_empty() {
        let append = format!(" for Symbol {}.", sym.symbol);
        msg.push_str(&append);
    }
    Ok(msg)
}

///
/// Handle the request using HTTP POST method like
/// `curl localhost:8080/tail/123 -d '{"symbol": "APPLE"}'`
///
async fn handle_post_request(mut req: Request<()>) -> tide::Result {
    let n: usize = req.param("n")?.parse().unwrap_or(0);
    let Symbol { symbol } = req.body_json().await?;
    Ok(format!("Requested symbol {} and a total of {} records", symbol, n).into())
}
