use actix_cors::Cors;
use actix_web::{get, web, App, HttpServer, Responder};
use dotenv::dotenv;
use odbc_api::{Environment, ConnectionOptions};
use serde::{Deserialize, Serialize};
use std::env;
use odbc_api::Cursor;

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

async fn fetch_latest_block() -> Result<BlockData, Box<dyn std::error::Error>> {
    dotenv().ok();
    let connection_string = env::var("ODBC_CONNECTION_STRING").expect("ODBC_CONNECTION_STRING must be set");

    let environment = Environment::new()?;
    let connection = environment.connect_with_connection_string(&connection_string, ConnectionOptions::default())?;

    let sql = r#"
        SELECT block_height, transaction_count, recent_transactions,
               average_fee, total_volume, difficulty, hash_rate, market_price, mempool_size
        FROM block_data
        ORDER BY block_height DESC
        LIMIT 1
    "#;

    if let Some(cursor) = connection.execute(&sql, ())? {
        let mut buffers = cursor.bind_buffer(10)?; // Adjust buffer size as needed

        while let Some(batch) = buffers.fetch()? {
            for row_index in 0..batch.num_rows() {
                let block_height: i32 = batch.at_as_str(0, row_index)?.parse()?;
                let transaction_count: i32 = batch.at_as_str(1, row_index)?.parse()?;
                let recent_transactions_str = batch.at_as_str(2, row_index)?;
                let recent_transactions: Vec<Transaction> = serde_json::from_str(recent_transactions_str).unwrap_or_default();

                return Ok(BlockData {
                    block_height,
                    transaction_count,
                    recent_transactions,
                    average_fee: batch.at_as_str(3, row_index)?.parse()?,
                    total_volume: batch.at_as_str(4, row_index)?.parse()?,
                    difficulty: batch.at_as_str(5, row_index)?.parse()?,
                    hash_rate: batch.at_as_str(6, row_index)?.parse()?,
                    market_price: batch.at_as_str(7, row_index)?.parse()?,
                    mempool_size: batch.at_as_str(8, row_index)?.parse()?,
                });
            }
        }
    }

    Err("No data found".into())
}

#[get("/latest_block")]
async fn get_latest_block() -> impl Responder {
    match fetch_latest_block().await {
        Ok(block_data) => web::Json(block_data),
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
    HttpServer::new(|| {
        App::new()
            .wrap(Cors::permissive())
            .service(get_latest_block)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}