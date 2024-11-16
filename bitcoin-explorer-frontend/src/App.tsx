import React, { useState, useEffect } from 'react';
import axios from 'axios';
import { Container, Row, Col, Card } from 'react-bootstrap';
import './App.css';

interface Transaction {
  hash: string;
  fee: number;
}

interface BlockData {
  block_height: number;
  transaction_count: number;
  total_volume: number;
  market_price: number;
  mempool_size: number;
  average_fee: number;
  recent_transactions: Transaction[];
}

function App() {
  const [blockData, setBlockData] = useState<BlockData | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const response = await axios.get<BlockData>('https://bitcoin-explorer-backend.onrender.com/latest_block');
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
    <Container className="mt-5">
      <h1 className="text-center text-primary mb-4">Bitcoin Block Explorer</h1>
      
      <Row className="mb-4">
        <Col md={4}>
          <Card className="info-card shadow-sm">
            <Card.Body>
              <Card.Title>Latest Block</Card.Title>
              <Card.Text>{blockData.block_height}</Card.Text>
            </Card.Body>
          </Card>
        </Col>
        <Col md={4}>
          <Card className="info-card shadow-sm">
            <Card.Body>
              <Card.Title>Transaction Count</Card.Title>
              <Card.Text>{blockData.transaction_count}</Card.Text>
            </Card.Body>
          </Card>
        </Col>
        <Col md={4}>
          <Card className="info-card shadow-sm">
            <Card.Body>
              <Card.Title>Total Volume</Card.Title>
              <Card.Text>{blockData.total_volume.toFixed(2)} BTC</Card.Text>
            </Card.Body>
          </Card>
        </Col>

        <Col md={4}>
          <Card className="info-card shadow-sm">
            <Card.Body>
              <Card.Title>Market Price</Card.Title>
              <Card.Text>${blockData.market_price}</Card.Text>
            </Card.Body>
          </Card>
        </Col>

        <Col md={4}>
          <Card className="info-card shadow-sm">
            <Card.Body>
              <Card.Title>Average Fee</Card.Title>
              <Card.Text>{blockData.average_fee.toFixed(3)}K Sats</Card.Text>
            </Card.Body>
          </Card>
        </Col>

        <Col md={4}>
          <Card className="info-card shadow-sm">
            <Card.Body>
              <Card.Title>Mempool Size</Card.Title>
              <Card.Text>{blockData.mempool_size}</Card.Text>
            </Card.Body>
          </Card>
        </Col>
      </Row>

      <h3 className="text-primary mb-3">Recent Transactions:</h3>

      {blockData.recent_transactions.map((tx) => (
        <Row key={tx.hash} className="mb-2">
          <Col>
            <div className="transaction-item p-3 shadow-sm rounded bg-light d-flex justify-content-between align-items-center">
              {tx.hash}
              <span className="text-success">{tx.fee} satoshis</span>
            </div>
          </Col>
        </Row>
      ))}
    </Container>
  );
}

export default App;