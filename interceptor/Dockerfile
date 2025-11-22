# Multi-stage Dockerfile for Int3rceptor
# Optimized for production deployment

# Stage 1: Build Rust backend
FROM rust:1.75-slim as rust-builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo files for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY core/Cargo.toml ./core/
COPY api/Cargo.toml ./api/
COPY cli/Cargo.toml ./cli/

# Build dependencies (cached layer)
RUN mkdir -p core/src api/src cli/src && \
    echo "fn main() {}" > core/src/lib.rs && \
    echo "fn main() {}" > api/src/lib.rs && \
    echo "fn main() {}" > cli/src/main.rs && \
    cargo build --release && \
    rm -rf core/src api/src cli/src

# Copy source code
COPY core ./core
COPY api ./api
COPY cli ./cli

# Build the application
RUN cargo build --release --bin interceptor

# Stage 2: Build Vue.js frontend
FROM node:20-slim as frontend-builder

WORKDIR /build

# Copy package files for dependency caching
COPY ui/package.json ui/package-lock.json ./

# Install dependencies (cached layer)
RUN npm ci --only=production

# Copy source code
COPY ui ./

# Build the frontend
RUN npm run build

# Stage 3: Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 -s /bin/bash interceptor

WORKDIR /app

# Copy built binaries from rust-builder
COPY --from=rust-builder /build/target/release/interceptor /app/

# Copy built frontend from frontend-builder
COPY --from=frontend-builder /build/dist /app/ui/dist

# Create data directory
RUN mkdir -p /app/data && chown -R interceptor:interceptor /app

# Switch to non-root user
USER interceptor

# Expose ports
EXPOSE 8080 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Environment variables
ENV INTERCEPTOR_DB_PATH=/app/data/interceptor.sqlite
ENV INTERCEPTOR_MAX_CONCURRENCY=128
ENV RUST_LOG=info

# Run the application
CMD ["/app/interceptor", "--listen", "0.0.0.0:8080", "--api", "0.0.0.0:3000"]
