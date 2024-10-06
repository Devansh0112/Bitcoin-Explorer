import React, { useState, useEffect } from 'react';
import axios from 'axios';
import './App.css';

interface Transaction {
  hash: string;
  fee: number;
}

interface BlockData {
  block_height: number;
  transaction_count: number;
  recent_transactions: Transaction[];
}

function App() {
  const [blockData, setBlockData] = useState<BlockData | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const response = await axios.get<BlockData>('http://localhost:8080/latest_block');
        setBlockData(response.data);
      } catch (error) {
        console.error('Error fetching data:', error);
      }
    };

    fetchData();
    const interval = setInterval(fetchData, 60000); // Fetch every minute

    return () => clearInterval(interval);
  }, []);

  if (!blockData) {
    return <div>Loading...</div>;
  }

  return (
    <div className="App">
      <h1>Bitcoin Explorer</h1>
      <h2>Latest Block: {blockData.block_height}</h2>
      <p>Total Transactions: {blockData.transaction_count}</p>
      <h3>Recent Transactions:</h3>
      <ul>
        {blockData.recent_transactions.map((tx) => (
          <li key={tx.hash}>
            Hash: {tx.hash} | Fee: {tx.fee} BTC
          </li>
        ))}
      </ul>
    </div>
  );
}

export default App;