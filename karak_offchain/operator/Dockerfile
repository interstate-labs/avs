# First stage: Builder
FROM rust:1.83-slim as builder

WORKDIR /app

# Install dependencies - including build-essential
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev git build-essential && \
    rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./

# Modify workspace to only include operator
RUN sed -i 's/members = \["aggregator", "operator"\]/members = ["operator"]/' Cargo.toml || true

# Create operator directory and copy its Cargo.toml
COPY operator/Cargo.toml ./operator/

# Create dummy source file to build dependencies
RUN mkdir -p ./operator/src && \
    echo "fn main() {}" > ./operator/src/main.rs

# Build dependencies only (without musl target)
RUN cargo build --release

# Remove the dummy files
RUN rm -rf ./operator/src

# Copy real source code
COPY operator/src ./operator/src
COPY abi ./abi

# Build the application (without musl target)
RUN cargo build --release

# Second stage: Runtime
FROM debian:bookworm-slim

WORKDIR /app

# Install curl for healthcheck
RUN apt-get update && \
    apt-get install -y ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/square-number-dss-operator /app/square-number-dss-operator

# Copy ABI files for runtime use
COPY --from=builder /app/abi /app/abi

# Set executable permissions
RUN chmod +x /app/square-number-dss-operator

# Command to run the application
CMD ["/app/square-number-dss-operator"]