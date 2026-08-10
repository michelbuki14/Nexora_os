# Build stage
FROM rust:1.80-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests
COPY Cargo.toml Cargo.lock ./
COPY services/common/Cargo.toml services/common/Cargo.toml
COPY services/api-gateway/Cargo.toml services/api-gateway/Cargo.toml
COPY services/tenant-service/Cargo.toml services/tenant-service/Cargo.toml
COPY services/audit-service/Cargo.toml services/audit-service/Cargo.toml
COPY services/retail-service/Cargo.toml services/retail-service/Cargo.toml
COPY services/fintech-service/Cargo.toml services/fintech-service/Cargo.toml
COPY services/gov-service/Cargo.toml services/gov-service/Cargo.toml

# Create dummy source files to cache dependencies
RUN mkdir -p services/common/src services/api-gateway/src services/tenant-service/src \
    services/audit-service/src services/retail-service/src services/fintech-service/src \
    services/gov-service/src && \
    for d in services/*/src; do echo "fn main() {}" > $d/main.rs; done

# Build dependencies
RUN cargo build --workspace --release --bin aos-api-gateway

# Copy actual source
COPY services/common/src services/common/src
COPY services/api-gateway/src services/api-gateway/src
COPY services/tenant-service/src services/tenant-service/src
COPY services/audit-service/src services/audit-service/src
COPY services/retail-service/src services/retail-service/src
COPY services/fintech-service/src services/fintech-service/src
COPY services/gov-service/src services/gov-service/src

# Build application
RUN cargo build --workspace --release --bin aos-api-gateway

# Runtime stage
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r aos && useradd -r -g aos aos

# Copy binary
COPY --from=builder /app/target/release/aos-api-gateway /usr/local/bin/aos-api-gateway

# Set ownership
RUN chown aos:aos /usr/local/bin/aos-api-gateway

USER aos

EXPOSE 3000

ENTRYPOINT ["aos-api-gateway"]