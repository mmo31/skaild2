# Build stage
FROM rust:slim AS builder

WORKDIR /app

# Copy all source files
COPY Cargo.toml ./
COPY migrations migrations
COPY crates crates

# Build the gateway binary
RUN cargo build --release -p gateway

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/gateway /app/gateway

EXPOSE 80 443

CMD ["/app/gateway"]
