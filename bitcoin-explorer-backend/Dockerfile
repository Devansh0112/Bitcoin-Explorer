FROM rust:bookworm as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

# Switch to a newer Debian version with the required GLIBC version
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libpq-dev ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/app/target/release/bitcoin-explorer-backend /usr/local/bin/
CMD ["bitcoin-explorer-backend"]
