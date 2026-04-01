# Multi-stage Dockerfile for IOU-Modern Rust Application
# Haven+ compliant: minimal base, non-root user, security scanning

# Stage 1: Build
FROM rust:1.75-bookworm-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy Cargo manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy main to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer is cached)
RUN cargo build --release && rm -rf src

# Copy source code
COPY crates ./crates
COPY migrations ./migrations

# Build the application
RUN cargo build --release && strip /build/target/release/iou-api

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies only
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user for security (Haven+ requirement)
RUN groupadd -r appuser -g 1001 && \
    useradd -r -u 1001 -g appuser appuser

WORKDIR /app

# Copy binaries from builder
COPY --from=builder /build/target/release/iou-api /app/
COPY --from=builder /build/target/release/iou-ai /app/

# Copy migrations
COPY --from=builder /build/migrations /app/migrations/

# Create directories
RUN mkdir -p /app/data /app/logs && \
    chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 8080

# Health check (Haven+ requirement)
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Set environment variables
ENV RUST_LOG=info \
    LOG_FORMAT=json \
    LOG_LEVEL=info

# Run the application
ENTRYPOINT ["/app/iou-api"]
