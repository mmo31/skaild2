# Build stage
FROM rust:1.75-slim as builder

WORKDIR /app

# Copy workspace files
COPY Cargo.toml ./
COPY crates/shared crates/shared
COPY crates/gateway crates/gateway
COPY crates/control-plane crates/control-plane

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
