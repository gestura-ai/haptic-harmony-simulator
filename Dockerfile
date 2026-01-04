# Multi-stage Dockerfile for Haptic Harmony Simulation
# Supports both CLI and GUI builds with cross-platform compilation

# Build stage for Rust application
FROM rust:1.75-slim as rust-builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libudev-dev \
    libdbus-1-dev \
    build-essential \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

# Build the CLI application
RUN cargo build --release

# Build stage for Node.js frontend
FROM node:18-slim as frontend-builder

WORKDIR /app/ui

# Copy package files
COPY ui/package*.json ./

# Install dependencies
RUN npm ci

# Copy frontend source
COPY ui/ ./

# Build frontend
RUN npm run build

# Runtime stage for CLI
FROM debian:bookworm-slim as cli-runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libudev1 \
    libdbus-1-3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false appuser

# Copy binary
COPY --from=rust-builder /app/target/release/haptic-harmony-simulation /usr/local/bin/

# Set ownership
RUN chown appuser:appuser /usr/local/bin/haptic-harmony-simulation

USER appuser

EXPOSE 8080

CMD ["haptic-harmony-simulation", "--mode", "cli"]

# Development stage with all tools
FROM rust:1.75-slim as development

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libudev-dev \
    libdbus-1-dev \
    libwebkit2gtk-4.0-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    build-essential \
    curl \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js
RUN curl -fsSL https://deb.nodesource.com/setup_18.x | bash - \
    && apt-get install -y nodejs

# Install development tools
RUN cargo install cargo-watch cargo-tarpaulin cargo-audit

# Set working directory
WORKDIR /app

# Copy source code
COPY . .

# Install frontend dependencies
RUN cd ui && npm install

# Expose ports for development
EXPOSE 8080 3000 1420

CMD ["cargo", "watch", "-x", "run -- --mode cli"]

# Production GUI stage (requires X11 forwarding)
FROM ubuntu:22.04 as gui-runtime

# Install GUI runtime dependencies
RUN apt-get update && apt-get install -y \
    libwebkit2gtk-4.1-0 \
    libxdo3 \
    libayatana-appindicator3-1 \
    librsvg2-2 \
    libudev1 \
    libdbus-1-3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false appuser

# Copy binary and frontend
COPY --from=rust-builder /app/target/release/haptic-harmony-simulation /usr/local/bin/
COPY --from=frontend-builder /app/ui/dist/ /app/ui/dist/

# Set ownership
RUN chown -R appuser:appuser /app /usr/local/bin/haptic-harmony-simulation

USER appuser

EXPOSE 8080

# Note: GUI mode requires X11 forwarding
# Run with: docker run -e DISPLAY=$DISPLAY -v /tmp/.X11-unix:/tmp/.X11-unix
CMD ["haptic-harmony-simulation", "--mode", "gui"]
