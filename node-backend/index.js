const express = require('express');
const dotenv = require('dotenv');
const sql = require('mssql');
const cors = require('cors');

dotenv.config();

const app = express();
app.use(express.json());
app.use(cors());

// MSSQL Configuration
const sqlConfig = {
  user: process.env.DB_USER,
  password: process.env.DB_PASSWORD,
  database: process.env.DB_NAME,
  server: process.env.DB_SERVER,
  pool: {
    max: 10,
    min: 0,
    idleTimeoutMillis: 30000
  },
  options: {
    encrypt: true, // Use true for Azure
    trustServerCertificate: false // Set to true for local dev/self-signed certs
  }
};

app.get('/latest_block', async (req, res) => {
  try {
    await sql.connect(sqlConfig);
    const result = await sql.query(`
      SELECT TOP 1 block_height, transaction_count, recent_transactions,
             average_fee, total_volume, difficulty, hash_rate, market_price, mempool_size
      FROM blockchain_data
      ORDER BY block_height DESC
    `);

    if (result.recordset.length > 0) {
      const row = result.recordset[0];
      let recent_transactions = [];

      try {
        // Ensure recent_transactions is valid JSON
        recent_transactions = JSON.parse(row.recent_transactions) || [];
      } catch (jsonError) {
        console.error("Failed to parse recent transactions:", jsonError);
        // Handle the error or set a default value
        recent_transactions = [];
      }

      const blockData = {
        block_height: row.block_height,
        transaction_count: row.transaction_count,
        recent_transactions,
        average_fee: row.average_fee,
        total_volume: row.total_volume,
        difficulty: row.difficulty,
        hash_rate: row.hash_rate,
        market_price: row.market_price,
        mempool_size: row.mempool_size,
      };
      res.json(blockData);
    } else {
      res.status(404).json({ error: 'No data found' });
    }
  } catch (error) {
    console.error(error);
    res.status(500).json({ error: 'An error occurred' });
  }
});

const PORT = process.env.PORT || 8080;
app.listen(PORT, () => {
  console.log(`Server running at http://localhost:${PORT}`);
});