use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::Row;

use crate::types::{
    answer::{Answer, AnswerId},
    question::{Question, QuestionId},
};

#[derive(Debug, Clone)]
pub struct Store {
    pub connection: PgPool,
}

impl Store {
    pub async fn new(db_url: &str) -> Self {
        let db_pool = match PgPoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await
        {
            Ok(pool) => pool,
            Err(e) => panic!("Couldn't establish DB connection: {}", e),
        };

        Store {
            connection: db_pool,
        }
    }
}
