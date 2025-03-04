FROM rust:latest as builder

WORKDIR /app

# Install system dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev build-essential git && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Copy the entire project
COPY . .

# Show Rust version for debugging
RUN rustc --version

# Build the application
RUN cargo build --release

# Create a smaller runtime image
FROM debian:bullseye-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libssl-dev && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Create logs directory
RUN mkdir -p /app/logs

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/symbiotic-restaking /app/symbiotic-restaking

# Run the application with the FetchEvents command
CMD ["/app/symbiotic-restaking", "FetchEvents"]