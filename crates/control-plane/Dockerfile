# Build stage
FROM rust:1.75-slim as builder

WORKDIR /app

# Copy workspace files
COPY Cargo.toml ./
COPY crates/shared crates/shared
COPY crates/control-plane crates/control-plane
COPY crates/gateway crates/gateway

# Build the control-plane binary
RUN cargo build --release -p control-plane

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/control-plane /app/control-plane

EXPOSE 8080

CMD ["/app/control-plane"]
