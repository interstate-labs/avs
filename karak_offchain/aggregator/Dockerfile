FROM rust:1.83-slim AS builder

WORKDIR /app

COPY . .

WORKDIR /app/aggregator

# Install necessary dependencies in a single step
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
        ca-certificates \
        openssl \
        libterm-readline-perl-perl && \
    rm -rf /var/lib/apt/lists/*

RUN cargo build --release --bin square-number-dss-aggregator

# ---- Runtime stage ----
FROM debian:bookworm-slim

WORKDIR /app

# Set environment to avoid interactive prompts
ENV DEBIAN_FRONTEND=noninteractive

# Install necessary runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends curl ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/square-number-dss-aggregator /app/square-number-dss-aggregator
COPY --from=builder /app/contracts /app/contracts

ENTRYPOINT ["/app/square-number-dss-aggregator"]
