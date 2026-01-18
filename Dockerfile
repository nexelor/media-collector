# Build stage
FROM rust:1.83-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build for release
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 appuser

# Create necessary directories
RUN mkdir -p /app/pictures /app/logs && \
    chown -R appuser:appuser /app

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/media-collector /app/media-collector

# Copy config template (will be overridden by mounted config)
COPY config-exemple.toml /app/config-exemple.toml

# Change ownership
RUN chown -R appuser:appuser /app

# Switch to app user
USER appuser

# Expose API port
EXPOSE 3000

# Run the application
CMD ["/app/media-collector"]