use actix_cors::Cors;
use actix_web::{get, web, App, HttpServer, Responder};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use std::env;

#[derive(Deserialize, Serialize, Debug)]
struct BlockData {
    block_height: i32,
    transaction_count: i32,
    recent_transactions: Vec<Transaction>,
    average_fee: f64,
    total_volume: f64,
    difficulty: f64,
    hash_rate: f64,
    market_price: f64,
    mempool_size: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Transaction {
    hash: String,
    fee: i64,
}

#[derive(sqlx::FromRow)]
struct RawBlockData {
    block_height: i32,
    transaction_count: i32,
    recent_transactions: serde_json::Value,
    average_fee: Option<f64>,
    total_volume: Option<f64>,
    difficulty: Option<f64>,
    hash_rate: Option<f64>,
    market_price: Option<f64>,
    mempool_size: Option<i32>,
}

#[get("/latest_block")]
async fn get_latest_block(pool: web::Data<PgPool>) -> impl Responder {
    let result = sqlx::query_as!(
        RawBlockData,
        r#"
        SELECT block_height, transaction_count, recent_transactions, 
               average_fee, total_volume, difficulty, hash_rate, market_price, mempool_size
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
                average_fee: raw_data.average_fee.unwrap_or(0.0),
                total_volume: raw_data.total_volume.unwrap_or(0.0),
                difficulty: raw_data.difficulty.unwrap_or(0.0),
                hash_rate: raw_data.hash_rate.unwrap_or(0.0),
                market_price: raw_data.market_price.unwrap_or(0.0),
                mempool_size: raw_data.mempool_size.unwrap_or(0),
            };
            web::Json(block_data)
        },
        Err(_) => web::Json(BlockData {
            block_height: 0,
            transaction_count: 0,
            recent_transactions: vec![],
            average_fee: 0.0,
            total_volume: 0.0,
            difficulty: 0.0,
            hash_rate: 0.0,
            market_price: 0.0,
            mempool_size: 0,
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