# Lightwalletd build for regtest
FROM golang:1.21-bookworm as builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    git \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Clone lightwalletd
WORKDIR /build
RUN git clone https://github.com/zcash/lightwalletd.git
WORKDIR /build/lightwalletd

# Build
RUN make

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    netcat-openbsd \
    && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=builder /build/lightwalletd/lightwalletd /usr/local/bin/lightwalletd
RUN chmod +x /usr/local/bin/lightwalletd

# Create directories
RUN mkdir -p /var/lightwalletd

# Copy entrypoint
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

WORKDIR /var/lightwalletd

EXPOSE 9067

ENTRYPOINT ["/entrypoint.sh"]