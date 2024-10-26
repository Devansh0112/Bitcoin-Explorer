# Stage 1: Build the Rust project
FROM rust:bookworm as builder

# Install ODBC libraries
RUN apt-get update && apt-get install -y unixodbc unixodbc-dev

# Set up working directory
WORKDIR /usr/src/app

# Copy source code into container
COPY . .

# Build the Rust project in release mode
RUN cargo build --release

# Stage 2: Create a smaller image with only the compiled binary and necessary libraries
FROM debian:bookworm-slim

# Install runtime dependencies (ODBC libraries)
RUN apt-get update && apt-get install -y \
    libpq-dev \
    ca-certificates \
    openssl \
    unixodbc \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/bitcoin_ingestion /usr/local/bin/

# Set the entry point to run the binary
CMD ["bitcoin_ingestion"]