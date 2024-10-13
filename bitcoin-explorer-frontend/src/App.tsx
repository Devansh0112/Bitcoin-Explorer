import React, { useState, useEffect } from 'react';
import axios from 'axios';

import './App.css'

interface Transaction {
  hash: string;
  fee: number;
}

interface BlockData {
  block_height: number;
  transaction_count: number;
  recent_transactions: Transaction[];
  average_fee: number;
  total_volume: number;
  difficulty: number;
  hash_rate: number;
  market_price: number;
  mempool_size: number;
}

const BlockExplorer: React.FC = () => {
  const [blockData, setBlockData] = useState<BlockData | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        setLoading(true);
        const response = await axios.get<BlockData>('http://localhost:8080/latest_block');
        setBlockData(response.data);
        setError(null);
      } catch (err) {
        setError('Failed to fetch block data');
        console.error(err);
      } finally {
        setLoading(false);
      }
    };

    fetchData();
    const interval = setInterval(fetchData, 60000); // Fetch every minute

    return () => clearInterval(interval);
  }, []);

  if (loading) return <div>Loading...</div>;
  if (error) return <div>Error: {error}</div>;
  if (!blockData) return <div>No data available</div>;

  return (
    <div className="block-explorer">
      <h1>Bitcoin Block Explorer</h1>
      <div className="block-info">
        <div className="info-item">
          <h3>Latest Block</h3>
          <p>{blockData.block_height}</p>
        </div>
        <div className="info-item">
          <h3>Transaction Count</h3>
          <p>{blockData.transaction_count}</p>
        </div>
        <div className="info-item">
          <h3>Average Fee</h3>
          <p>{blockData.average_fee.toFixed(3)}K Sats</p>
        </div>
        <div className="info-item">
          <h3>Total Volume</h3>
          <p>{blockData.total_volume.toFixed(3)} BTC</p>
        </div>
        <div className="info-item">
          <h3>Market Price</h3>
          <p>${blockData.market_price.toFixed(2)}</p>
        </div>
        <div className="info-item">
          <h3>Mempool Size</h3>
          <p>{blockData.mempool_size}</p>
        </div>
      </div>
      <div className="recent-transactions">
        <h2>Recent Transactions</h2>
        <ul>
          {blockData.recent_transactions.map((tx, index) => (
            <li key={index}>
              <span className="transaction-hash">{tx.hash}</span>
              <span className="transaction-fee">{tx.fee} satoshis</span>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
};

export default BlockExplorer;