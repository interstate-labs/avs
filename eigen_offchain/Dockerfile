
# Use the official Rust image as base
FROM rust:1.81-slim-bullseye

# Create a working directory
WORKDIR /usr/src/app

# Install required system dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml ./

# Copy the source code
COPY src/ ./src/

# Build the application
RUN cargo build --release

# Command to run the application
CMD ["cargo", "run", "--release"]