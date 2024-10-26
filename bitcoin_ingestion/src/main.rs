use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize};
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;
use reqwest;
use std::error::Error as StdError;
use dotenv::dotenv;
use std::env;
use odbc_api::{Environment, ConnectionOptions};

#[derive(Deserialize, Debug)]
struct BlockData {
    block_height: i32,
    transaction_count: i32,
    average_fee: f64,
    total_volume: f64,
    difficulty: f64,
    hash_rate: f64,
    market_price: f64,
    trading_volume_24h: i64,
    active_addresses_24h: i64,
    mempool_size: f64,
}

#[derive(Deserialize, Debug)]
struct RawBlockData {
    x: BlockInfo,
}

#[derive(Deserialize, Debug)]
struct BlockInfo {
    hash: String,
}

async fn update_database(connection_string: &str, block_data: &BlockData) -> Result<(), Box<dyn StdError>> {
    let environment = Environment::new()?;
    
    // Connect to Azure SQL Database using ODBC with connection options
    let connection_options = ConnectionOptions::default();
    let connection = environment.connect_with_connection_string(connection_string, connection_options)?;

    // INSERT statement for adding new records
    let sql_query = r#"
        INSERT INTO block_data (block_height, transaction_count, average_fee, total_volume, difficulty, hash_rate, market_price, trading_volume_24h, active_addresses_24h, mempool_size)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
    "#;

    // Prepare individual parameters
    let block_height = block_data.block_height;
    let transaction_count = block_data.transaction_count;
    
    let average_fee = block_data.average_fee;
    let total_volume = block_data.total_volume;
    let difficulty = block_data.difficulty;
    let hash_rate = block_data.hash_rate;
    let market_price = block_data.market_price;
    let trading_volume_24h = block_data.trading_volume_24h;
    let active_addresses_24h = block_data.active_addresses_24h;
    let mempool_size = block_data.mempool_size;

    // Execute the statement with bound parameters
    connection.execute(
        &sql_query,
            (&block_height,
            &transaction_count,
            &average_fee,
            &total_volume,
            &difficulty,
            &hash_rate,
            &market_price,
            &trading_volume_24h,
            &active_addresses_24h,
            &mempool_size),
     )?;

     println!("New block data inserted into the database successfully.");

     Ok(())
}

async fn run_websocket(connection_string: &str) -> Result<(), Box<dyn StdError>> {
   let (ws_stream, _) = connect_async(Url::parse("wss://ws.blockchain.info/inv")?).await?;
   let (mut write, read) = ws_stream.split();

   write.send(Message::Text(serde_json::to_string(&json!({"op": "blocks_sub"}))?)).await?;
   let mut read = read.map(|message| message);
   let http_client = reqwest::Client::new();

   while let Some(message_result) = read.next().await {
       match message_result {
           Ok(message) => {
               if let Message::Text(text) = message {
                   let raw_data: RawBlockData = serde_json::from_str(&text)?;

                   // Fetch block details with HTTP request
                   let block_url = format!("https://api.blockchain.info/rawblock/{}", raw_data.x.hash);
                   let block_response = http_client.get(&block_url).send().await?.json::<BlockData>().await?;

                   update_database(connection_string, &block_response).await?;
                   println!("Processed new block data: {:?}", block_response);
               }
           }
           Err(e) => {
               eprintln!("WebSocket error: {:?}", e);
               break;
           }
       }
   }

   Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
   dotenv().ok();

   // Get ODBC connection string from environment variables
   let connection_string = env::var("ODBC_CONNECTION_STRING").expect("ODBC_CONNECTION_STRING must be set");
   println!("{connection_string}");

   // Run WebSocket handling loop
   loop {
       match run_websocket(&connection_string).await {
           Ok(_) => println!("WebSocket connection closed normally"),
           Err(e) => {
               eprintln!("WebSocket error: {:?}", e);
               tokio::time::sleep(std::time::Duration::from_secs(5)).await;
               println!("Attempting to reconnect...");
           }
       }
   }
}