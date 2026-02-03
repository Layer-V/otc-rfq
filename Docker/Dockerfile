# =============================================================================
# OTC-RFQ Service - Production Dockerfile
# =============================================================================
# Multi-stage build for optimized production image
# Uses cargo-chef for dependency caching
# =============================================================================

# -----------------------------------------------------------------------------
# Stage 1: Chef - Prepare dependency recipe
# -----------------------------------------------------------------------------
FROM rust:1.83-slim-bookworm AS chef

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-chef --locked

WORKDIR /app

# -----------------------------------------------------------------------------
# Stage 2: Planner - Create dependency recipe
# -----------------------------------------------------------------------------
FROM chef AS planner

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# -----------------------------------------------------------------------------
# Stage 3: Builder - Build dependencies and application
# -----------------------------------------------------------------------------
FROM chef AS builder

# Copy recipe and build dependencies first (cached layer)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source code and build application
COPY . .
RUN cargo build --release --bin otc-rfq

# Strip debug symbols for smaller binary
RUN strip /app/target/release/otc-rfq || true

# -----------------------------------------------------------------------------
# Stage 4: Runtime - Minimal production image
# -----------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN groupadd --gid 1000 otc \
    && useradd --uid 1000 --gid otc --shell /bin/bash --create-home otc

# Create directories for configuration and data
RUN mkdir -p /app/config /app/data /app/logs \
    && chown -R otc:otc /app

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/otc-rfq /app/otc-rfq

# Set ownership
RUN chown otc:otc /app/otc-rfq

# Switch to non-root user
USER otc

# Environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1
ENV OTC_RFQ_CONFIG_PATH=/app/config
ENV OTC_RFQ_DATA_PATH=/app/data
ENV OTC_RFQ_LOG_PATH=/app/logs

# Expose ports
# HTTP API port
EXPOSE 8080
# gRPC port
EXPOSE 50051
# Metrics port
EXPOSE 9090

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Entrypoint with proper signal handling
ENTRYPOINT ["/app/otc-rfq"]

# Default command (can be overridden)
CMD ["--config", "/app/config/config.toml"]

# Labels for metadata
LABEL org.opencontainers.image.title="OTC-RFQ Service"
LABEL org.opencontainers.image.description="OTC Request for Quote Trading Platform"
LABEL org.opencontainers.image.vendor="Taunais"
LABEL org.opencontainers.image.source="https://github.com/joaquinbejar/otc-rfq"
