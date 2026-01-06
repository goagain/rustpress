# Multi-stage build for RustPress

# Stage 1: Build main frontend
FROM node:20-alpine AS main-frontend-builder

WORKDIR /app/frontend

# Copy frontend package files
COPY frontend/package*.json ./

# Install dependencies
RUN npm ci

# Copy frontend source
COPY frontend/ ./

# Build frontend
RUN npm run build

# Stage 2: Build admin frontend
FROM node:20-alpine AS admin-frontend-builder

WORKDIR /app/admin-frontend

# Copy admin frontend package files
COPY admin-frontend/package*.json ./

# Install dependencies
RUN npm ci

# Copy admin frontend source
COPY admin-frontend/ ./

# Build admin frontend
RUN npm run build

# Stage 3: Build Rust backend
FROM rust:1.92-alpine AS rust-builder

# Install build dependencies
RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static

WORKDIR /app

# Copy workspace Cargo files
COPY Cargo.toml Cargo.lock ./

# Copy WIT definitions
COPY wit ./wit

# Copy core package
COPY core ./core

# Copy migration crate
COPY rustpress-migration ./rustpress-migration

# Copy rustpress-cli (for building)
COPY rustpress-cli ./rustpress-cli

# Build the application in release mode (from workspace)
RUN cargo build --release -p rustpress-core

# Stage 3: Final runtime image
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    tzdata \
    wget \
    && update-ca-certificates

# Create app user
RUN addgroup -g 1000 appuser && \
    adduser -D -u 1000 -G appuser appuser

WORKDIR /app

# Copy built binary from rust-builder
COPY --from=rust-builder /app/target/release/rustpress /app/rustpress

# Copy frontend builds from builders
COPY --from=main-frontend-builder /app/frontend/dist /app/frontend/dist
COPY --from=admin-frontend-builder /app/admin-frontend/dist /app/admin-frontend/dist

# Create directories for uploads and plugins
RUN mkdir -p /app/uploads && \
    mkdir -p /app/installed_plugins && \
    chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 3000

# Set environment variables
ENV RUST_LOG=info
ENV DATABASE_URL=postgres://postgres:password@postgres:5432/rustpress
ENV STORAGE_DIR=/app/uploads
ENV STORAGE_BASE_URL=http://localhost:3000/uploads

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3000/api/health || exit 1

# Run the application
CMD ["./rustpress"]
