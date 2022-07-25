use chrono::prelude::*;
use serde::Serialize;
use std::collections::VecDeque;
use xactor::*;

///
/// xactor message
///
#[message]
#[derive(Clone)]
pub struct DataStreamingMsg {
    pub symbol: String,
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

#[message]
#[derive(Clone)]
pub struct ProcessQuoteMsg {
    pub symbol: String,
    pub closes: Vec<f64>,
    pub from: DateTime<Utc>,
}

#[message]
#[derive(Clone, Serialize)]
pub struct PrintableMsg {
    pub symbol: String,
    pub from: DateTime<Utc>,
    pub last_price: f64,
    pub pct_change: f64,
    pub period_max: f64,
    pub period_min: f64,
    pub sma: f64,
}

#[message(result = "VecDeque<PrintableMsg>")]
#[derive(Clone, Serialize)]
pub struct RequestMsg {
    pub sym: String,
    pub n: usize,
}
