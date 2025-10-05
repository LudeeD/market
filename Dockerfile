# Build stage
FROM rust:alpine AS builder

WORKDIR /app

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    pkgconfig

# Copy source code
COPY Cargo.toml Cargo.lock ./
COPY .sqlx ./.sqlx
COPY src ./src
COPY templates ./templates
COPY static ./static

# Build the application with offline mode for sqlx
ENV SQLX_OFFLINE=true
RUN cargo build --release

# Runtime stage
FROM alpine:latest

WORKDIR /app

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    libgcc

# Copy binary from builder
COPY --from=builder /app/target/release/market /app/market

# Copy templates and static files
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/static /app/static

# Create data directory for SQLite database
RUN mkdir -p /app/data

# Set environment variables
ENV DATABASE_URL=sqlite:/app/data/market.db
ENV HOST=0.0.0.0
ENV PORT=3000

# Expose port
EXPOSE 3000

# Run the application
CMD ["/app/market"]
