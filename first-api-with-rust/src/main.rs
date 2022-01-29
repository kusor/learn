use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use warp::Filter;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Question {
    id: String,
    title: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Store(HashMap<String, Question>);

impl Store {
    fn new() -> Self {
        Store { 0: Self::init() }
    }

    fn init() -> HashMap<String, Question> {
        let q = include_str!("../questions.json");
        serde_json::from_str(q).expect("Can't read json")
    }
}

async fn get_questions(param: String, store: Store) -> Result<impl warp::Reply, warp::Rejection> {
    println!("{:#}", param);
    Ok(warp::reply::json(&store))
}

#[tokio::main]
async fn main() {
    let store = Store::new();
    let store_filter = warp::any().map(move || store.clone());

    let get_questions = warp::get()
        .and(warp::path("questions"))
        .and(warp::query::raw())
        .and(warp::path::end())
        .and(store_filter)
        .and_then(get_questions);

    warp::serve(get_questions.with(warp::trace::request()))
        .run(([127, 0, 0, 1], 3030))
        .await;

    // GET /hello/warp => 200 OK with body "Hello, warp!"
    // let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!\n", name));

    // warp::serve(hello).run(([127, 0, 0, 1], 3030)).await;
}
