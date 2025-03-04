FROM rust:1.83-slim AS builder

# Set working directory in container
WORKDIR /app

# Copy workspace files
COPY karak_offchain/Cargo.toml karak_offchain/Cargo.lock ./

# Modify workspace members to only include aggregator
RUN sed -i 's/members = \["aggregator", "operator"\]/members = ["aggregator"]/' Cargo.toml

# Copy aggregator package
COPY karak_offchain/aggregator/Cargo.toml ./aggregator/
COPY karak_offchain/aggregator/src ./aggregator/src

# Copy ABI files
COPY karak_offchain/abi/ ./abi/

# Copy contracts
COPY onchain/contracts/ ./onchain/contracts/

# Install system dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
        ca-certificates \
        openssl && \
    rm -rf /var/lib/apt/lists/*

# Build only the aggregator
RUN cargo build --release --package square-number-dss-aggregator

# ---- Runtime stage ----
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        libssl3 \
        ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy built binary
COPY --from=builder \
    /app/target/release/square-number-dss-aggregator \
    /app/

# Copy contract artifacts
COPY --from=builder \
    /app/onchain/contracts \
    /app/onchain/contracts

ENTRYPOINT ["/app/square-number-dss-aggregator"]