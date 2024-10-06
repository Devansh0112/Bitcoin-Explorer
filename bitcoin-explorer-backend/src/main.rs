use actix_cors::Cors;
use actix_web::{get, web, App, HttpServer, Responder};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use std::env;

#[derive(Serialize, Deserialize)]
struct BlockData {
    block_height: i32,
    transaction_count: i32,
    recent_transactions: Vec<Transaction>,
}

#[derive(Serialize, Deserialize)]
struct Transaction {
    hash: String,
    fee: i64,
}

#[derive(sqlx::FromRow)]
struct RawBlockData {
    block_height: i32,
    transaction_count: i32,
    recent_transactions: serde_json::Value,
}

#[get("/latest_block")]
async fn get_latest_block(pool: web::Data<PgPool>) -> impl Responder {
    let result = sqlx::query_as!(
        RawBlockData,
        r#"
        SELECT block_height, transaction_count, recent_transactions
        FROM block_data
        ORDER BY block_height DESC
        LIMIT 1
        "#
    )
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(raw_data) => {
            let recent_transactions: Vec<Transaction> = serde_json::from_value(raw_data.recent_transactions).unwrap_or_default();
            let block_data = BlockData {
                block_height: raw_data.block_height,
                transaction_count: raw_data.transaction_count,
                recent_transactions,
            };
            web::Json(block_data)
        },
        Err(_) => web::Json(BlockData {
            block_height: 0,
            transaction_count: 0,
            recent_transactions: vec![],
        }),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::Data::new(pool.clone()))
            .service(get_latest_block)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}