# Bitcoin Explorer

A real-time Bitcoin blockchain data viewer built with Rust and React.

## Overview

This Bitcoin Explorer project fetches and displays live blockchain data, including the latest block height, transaction count, and recent transactions. It consists of three main components:

1. Data Ingestion Service (Rust)
2. Backend API (Rust with Actix-web)
3. Frontend Application (React with TypeScript)

## Features

- Real-time fetching of Bitcoin blockchain data
- Display of latest block height and transaction count
- List of 5 most recent transactions with their fees
- Automatic updates every minute

## Tech Stack

- **Backend**: Rust, Actix-web, SQLx
- **Frontend**: React, TypeScript, Axios
- **Database**: PostgreSQL
- **API**: Blockchain.com API

## Prerequisites

- Rust (latest stable version)
- Node.js and npm
- PostgreSQL
- Docker (optional)

## Setup and Installation

### 1. Database Setup

```
sql
CREATE DATABASE bitcoin_explorer;
CREATE TABLE block_data (
    block_height INTEGER PRIMARY KEY,
    transaction_count INTEGER NOT NULL,
    recent_transactions JSONB NOT NULL
);
```

### 2. Data Ingestion Service

cd data-ingestion
cp .env.example .env  # Edit with your database URL
cargo run

### 3. Backend API

cd backend
cp .env.example .env  # Edit with your database URL
cargo run

### 4. Frontend Application

cd frontend
npm install
npm start

## Usage
After starting all components, visit http://localhost:3000 in your web browser to view the Bitcoin Explorer interface.
Development
Data Ingestion: Modify data-ingestion/src/main.rs to change data fetching logic.
Backend API: Update backend/src/main.rs for new endpoints or data processing.
Frontend: Edit frontend/src/App.tsx to modify the UI or add features.

## Contributing
Contributions are welcome! Please feel free to submit a Pull Request.

## License
This project is licensed under the MIT License. See the LICENSE file for details.
