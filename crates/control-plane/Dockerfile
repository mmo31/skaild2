# Build stage
FROM rust:slim AS builder

WORKDIR /app

# Copy manifests for dependency caching
COPY Cargo.toml ./
COPY crates/shared/Cargo.toml crates/shared/Cargo.toml
COPY crates/control-plane/Cargo.toml crates/control-plane/Cargo.toml
COPY crates/gateway/Cargo.toml crates/gateway/Cargo.toml

# Copy migrations (needed by sqlx::migrate! macro at compile time)
COPY migrations migrations

# Create dummy source files to build dependencies
RUN mkdir -p crates/shared/src && echo "// dummy" > crates/shared/src/lib.rs && \
    mkdir -p crates/control-plane/src && echo "// dummy" > crates/control-plane/src/lib.rs && echo "fn main() {}" > crates/control-plane/src/main.rs && \
    mkdir -p crates/gateway/src && echo "fn main() {}" > crates/gateway/src/main.rs

# Build dependencies (this layer is cached)
RUN cargo build --release -p control-plane

# Copy real source code (overwrites dummy files)
COPY crates/shared crates/shared
COPY crates/control-plane crates/control-plane
COPY crates/gateway crates/gateway

# Remove dummy-compiled artifacts so cargo rebuilds our crates from real source.
# Touching Cargo.toml is not sufficient because Docker BuildKit preserves host
# file timestamps, making copied source files appear older than the dummy build.
RUN rm -f target/release/control-plane \
          target/release/deps/control_plane* \
          target/release/deps/libshared* \
    && rm -rf target/release/.fingerprint/control-plane-* \
              target/release/.fingerprint/shared-*

# Build only the application code (fast — dependencies are already cached above)
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
