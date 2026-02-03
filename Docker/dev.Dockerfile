# =============================================================================
# OTC-RFQ Service - Development Dockerfile
# =============================================================================
# Development image with hot-reload support via cargo-watch
# Includes all development dependencies and debug symbols
# =============================================================================

FROM rust:1.83-slim-bookworm

# Install development dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    curl \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install development tools
RUN cargo install cargo-watch --locked
RUN cargo install cargo-tarpaulin --locked || true

# Create non-root user for development
RUN groupadd --gid 1000 dev \
    && useradd --uid 1000 --gid dev --shell /bin/bash --create-home dev

# Create app directory
RUN mkdir -p /app && chown -R dev:dev /app

WORKDIR /app

# Switch to non-root user
USER dev

# Environment variables for development
ENV RUST_LOG=debug
ENV RUST_BACKTRACE=full
ENV CARGO_HOME=/home/dev/.cargo
ENV CARGO_TARGET_DIR=/app/target

# Expose ports
# HTTP API port
EXPOSE 8080
# gRPC port
EXPOSE 50051
# Metrics port
EXPOSE 9090

# Default command: watch for changes and rebuild
CMD ["cargo", "watch", "-x", "run"]

# Labels for metadata
LABEL org.opencontainers.image.title="OTC-RFQ Service (Development)"
LABEL org.opencontainers.image.description="OTC Request for Quote Trading Platform - Development Image"
