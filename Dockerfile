# Build stage
FROM rust:1.80-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests (one COPY per member to maximise layer cache hits)
COPY Cargo.toml Cargo.lock ./
COPY services/common/Cargo.toml services/common/Cargo.toml
COPY services/api-gateway/Cargo.toml services/api-gateway/Cargo.toml
COPY services/workforce-service/Cargo.toml services/workforce-service/Cargo.toml
COPY services/tenant-service/Cargo.toml services/tenant-service/Cargo.toml
COPY services/audit-service/Cargo.toml services/audit-service/Cargo.toml
COPY services/migrate/Cargo.toml services/migrate/Cargo.toml
COPY services/retail-service/Cargo.toml services/retail-service/Cargo.toml
COPY services/fintech-service/Cargo.toml services/fintech-service/Cargo.toml
COPY services/gov-service/Cargo.toml services/gov-service/Cargo.toml

# Create stub source files so cargo can resolve + cache all dependencies
# without copying real source (keeps this layer stable across source changes).
RUN mkdir -p \
    services/common/src \
    services/api-gateway/src \
    services/workforce-service/src \
    services/tenant-service/src \
    services/audit-service/src \
    services/migrate/src \
    services/retail-service/src \
    services/fintech-service/src \
    services/gov-service/src && \
    for d in services/*/src; do echo "fn main() {}" > "$d/main.rs"; done

# Dependency-only build (stubs compile, real code comes next)
RUN cargo build --workspace --release

# Copy real source for every workspace member
COPY services/common/src services/common/src
COPY services/api-gateway/src services/api-gateway/src
COPY services/workforce-service/src services/workforce-service/src
COPY services/tenant-service/src services/tenant-service/src
COPY services/audit-service/src services/audit-service/src
COPY services/migrate/src services/migrate/src
COPY services/retail-service/src services/retail-service/src
COPY services/fintech-service/src services/fintech-service/src
COPY services/gov-service/src services/gov-service/src

# Touch stubs so cargo sees real source as newer and re-links
RUN touch services/*/src/main.rs

# Full build
RUN cargo build --workspace --release

# Runtime stage
FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Non-root user
RUN groupadd -r aos && useradd -r -g aos aos

# Copy all service binaries
COPY --from=builder /app/target/release/aos-api-gateway /usr/local/bin/
COPY --from=builder /app/target/release/aos-workforce-service /usr/local/bin/
COPY --from=builder /app/target/release/aos-tenant-service /usr/local/bin/
COPY --from=builder /app/target/release/aos-audit-service /usr/local/bin/
COPY --from=builder /app/target/release/aos-migrate /usr/local/bin/

RUN chown aos:aos \
    /usr/local/bin/aos-api-gateway \
    /usr/local/bin/aos-workforce-service \
    /usr/local/bin/aos-tenant-service \
    /usr/local/bin/aos-audit-service \
    /usr/local/bin/aos-migrate

USER aos

EXPOSE 3000

# Default entrypoint is the gateway; override in docker-compose per service.
ENTRYPOINT ["aos-api-gateway"]
