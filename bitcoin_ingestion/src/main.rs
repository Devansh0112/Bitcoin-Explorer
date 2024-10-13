use futures_util::StreamExt;
use futures_util::SinkExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::PgPool;
use serde_json::Value;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tokio_tungstenite::tungstenite;
use url::Url;
use reqwest;
use std::error::Error;
use dotenv::dotenv;
use std::env;


#[derive(Deserialize, Serialize, Debug)]
struct BlockData {
    block_height: i32,
    transaction_count: i32,
    recent_transactions: String, // JSON string of recent transactions
    average_fee: f64,
    total_volume: f64,
    difficulty: f64,
    hash_rate: f64,
    market_price: f64,
}

#[derive(Deserialize, Debug)]
struct RawBlockData {
    x: BlockInfo,
}

#[derive(Deserialize, Debug)]
struct BlockInfo {
    height: i32,
    hash: String,
}

async fn update_database(pool: &PgPool, block_data: &BlockData) -> Result<(), sqlx::Error> {
    let recent_transactions: Value = serde_json::from_str(&block_data.recent_transactions)
        .map_err(|e| sqlx::Error::Protocol(format!("JSON parsing error: {}", e)))?;

    sqlx::query!(
        r#"
        INSERT INTO block_data 
        (block_height, transaction_count, recent_transactions, average_fee, total_volume, difficulty, hash_rate, market_price)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (block_height) DO UPDATE SET
        transaction_count = EXCLUDED.transaction_count,
        recent_transactions = EXCLUDED.recent_transactions,
        average_fee = EXCLUDED.average_fee,
        total_volume = EXCLUDED.total_volume,
        difficulty = EXCLUDED.difficulty,
        hash_rate = EXCLUDED.hash_rate,
        market_price = EXCLUDED.market_price
        "#,
        block_data.block_height,
        block_data.transaction_count,
        recent_transactions,  // Now this is a serde_json::Value
        block_data.average_fee,
        block_data.total_volume,
        block_data.difficulty,
        block_data.hash_rate,
        block_data.market_price
    )
    .execute(pool)
    .await?;

    Ok(())
}

fn calculate_average_fee(block_details: &serde_json::Value) -> f64 {
    let empty_vec = Vec::new();
    let transactions = block_details["tx"].as_array().unwrap_or(&empty_vec);
    let total_fee: f64 = transactions.iter()
        .map(|tx| tx["fee"].as_f64().unwrap_or(0.0))
        .sum();
    let tx_count = transactions.len() as f64;
    if tx_count > 0.0 { total_fee / tx_count } else { 0.0 }
}

fn calculate_total_volume(block_details: &serde_json::Value) -> f64 {
    let empty_vec = Vec::new();
    let transactions = block_details["tx"].as_array().unwrap_or(&empty_vec);
    transactions.iter()
        .flat_map(|tx| tx["out"].as_array().unwrap_or(&empty_vec))
        .map(|out| out["value"].as_f64().unwrap_or(0.0))
        .sum()
}

async fn run_websocket(pool: PgPool) -> Result<(), Box<dyn Error>> {
    let (ws_stream, _) = connect_async(Url::parse("wss://ws.blockchain.info/inv")?).await?;
    let (mut write, read) = ws_stream.split();

    // Subscribe to new blocks
    write.send(Message::Text(serde_json::to_string(&serde_json::json!({
        "op": "blocks_sub"
    }))?)).await?;

    let mut read = read.map(|message| message);
    let client = reqwest::Client::new();

    while let Some(message_result) = read.next().await {
        match message_result {
            Ok(message) => {
                if let Message::Text(text) = message {
                    // Your existing message handling code here
                    let raw_data: RawBlockData = serde_json::from_str(&text)?;
                    
                    // Fetch additional data using HTTP requests
                    let block_details: serde_json::Value = client.get(&format!(
                        "https://blockchain.info/rawblock/{}",
                        raw_data.x.hash
                    )).send().await?.json().await?;

                    let market_data: serde_json::Value = client.get(
                        "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd"
                    ).send().await?.json().await?;

                    let recent_transactions = serde_json::to_string(
                        &block_details["tx"]
                            .as_array()
                            .unwrap_or(&Vec::new())
                            .iter()
                            .take(5)
                            .map(|tx| {
                                json!({
                                    "fee": tx["fee"].as_i64().unwrap_or(0),
                                    "hash": tx["hash"].as_str().unwrap_or("").to_string(),
                                })
                            })
                            .collect::<Vec<_>>()
                    )?;
                    
                    let block_data = BlockData {
                        block_height: raw_data.x.height,
                        transaction_count: block_details["n_tx"].as_i64().unwrap_or(0) as i32,
                        recent_transactions,
                        average_fee: calculate_average_fee(&block_details),
                        total_volume: calculate_total_volume(&block_details),
                        difficulty: block_details["difficulty"].as_f64().unwrap_or(0.0),
                        hash_rate: block_details["difficulty"].as_f64().unwrap_or(0.0) / 600.0, // Approximate
                        market_price: market_data["bitcoin"]["usd"].as_f64().unwrap_or(0.0),
                    };

                    println!("Processed new block data: {:?}", block_data);

                    if let Err(e) = update_database(&pool, &block_data).await {
                        eprintln!("Error updating database: {}", e);
                    }
                }
            },
            Err(e) => {
                if let tungstenite::Error::Protocol(tungstenite::error::ProtocolError::ResetWithoutClosingHandshake) = e {
                    println!("Connection reset without closing handshake. Reconnecting...");
                    break;
                } else {
                    eprintln!("WebSocket error: {:?}", e);
                    break;
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await?;

    loop {
        match run_websocket(pool.clone()).await {
            Ok(_) => println!("WebSocket connection closed normally"),
            Err(e) => {
                eprintln!("WebSocket error: {:?}", e);
                if let Some(tungstenite_error) = e.downcast_ref::<tungstenite::Error>() {
                    match tungstenite_error {
                        tungstenite::Error::Protocol(tungstenite::error::ProtocolError::ResetWithoutClosingHandshake) => {
                            println!("Connection reset without closing handshake. Reconnecting...");
                        },
                        _ => {
                            eprintln!("Unexpected WebSocket error: {:?}", tungstenite_error);
                        }
                    }
                }
            }
        }

        // Wait before attempting to reconnect
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        println!("Attempting to reconnect...");
    }
}