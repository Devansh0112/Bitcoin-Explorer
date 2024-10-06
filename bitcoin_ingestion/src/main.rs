use dotenv::dotenv;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use std::env;
use tokio::time::{self, Duration};

#[derive(Deserialize, Debug)]
struct BlockInfo {
    height: i32,
    hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Transaction {
    hash: String,
    fee: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct BlockData {
    block_height: i32,
    transaction_count: i32,
    recent_transactions: Vec<Transaction>,
}

async fn fetch_block_data(client: &Client) -> Result<BlockData, reqwest::Error> {
    // Fetch latest block info
    let block_info: BlockInfo = client
        .get("https://blockchain.info/latestblock")
        .send()
        .await?
        .json()
        .await?;

    // Fetch block details
    let block_details: serde_json::Value = client
        .get(&format!(
            "https://blockchain.info/rawblock/{}",
            block_info.hash
        ))
        .send()
        .await?
        .json()
        .await?;

    // Get transaction count
    let transaction_count = block_details["tx"].as_array().unwrap().len() as i32;

    // Extract recent transactions
    let recent_transactions = block_details["tx"]
        .as_array()
        .unwrap()
        .iter()
        .take(5)
        .map(|tx| Transaction {
            hash: tx["hash"].as_str().unwrap().to_string(),
            fee: tx["fee"].as_i64().unwrap(),
        })
        .collect::<Vec<_>>();

    Ok(BlockData {
        block_height: block_info.height,
        transaction_count,
        recent_transactions,
    })
}

async fn update_block_data(pool: &PgPool, block_data: &BlockData) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO block_data (block_height, transaction_count, recent_transactions) 
         VALUES ($1, $2, $3) 
         ON CONFLICT (block_height) DO UPDATE SET 
         transaction_count = EXCLUDED.transaction_count, 
         recent_transactions = EXCLUDED.recent_transactions"
    )
    .bind(block_data.block_height)
    .bind(block_data.transaction_count)
    .bind(serde_json::to_value(&block_data.recent_transactions).unwrap())
    .execute(pool)
    .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await?;

    // Create the table if it doesn't exist
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS block_data (
            block_height INTEGER PRIMARY KEY,
            transaction_count INTEGER NOT NULL,
            recent_transactions JSONB NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    let client = Client::new();
    let mut interval = time::interval(Duration::from_secs(60));

    loop {
        interval.tick().await;

        match fetch_block_data(&client).await {
            Ok(block_data) => {
                println!("Fetched block data: {:?}", block_data);
                if let Err(e) = update_block_data(&pool, &block_data).await {
                    eprintln!("Error updating block data: {}", e);
                }
            }
            Err(e) => eprintln!("Error fetching block data: {}", e),
        }
    }
}