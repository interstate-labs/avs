# Use Alpine for a musl libc environment (for static linking)
FROM rust:alpine as builder

WORKDIR /app

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev pkgconfig git build-base

# Copy the entire project
COPY . .

# Build with musl for static linking
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl

# Use a minimal Alpine image for runtime
FROM alpine:latest

WORKDIR /app

# Install minimal runtime dependencies
RUN apk add --no-cache ca-certificates

# Create logs directory
RUN mkdir -p /app/logs

# Copy the statically linked binary from the builder stage
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/eigen_offchain /app/eigen_offchain

# Run the application with the FetchEvents command
CMD ["/app/eigen_offchain", "fetch-events"]