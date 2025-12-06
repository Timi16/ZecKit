FROM rust:1.85-slim-bookworm

# Install dependencies including netcat for health checks
RUN apt-get update && apt-get install -y \
    build-essential \
    gcc \
    protobuf-compiler \
    libssl-dev \
    curl \
    git \
    netcat-openbsd \
    && rm -rf /var/lib/apt/lists/*

# Clone YOUR FORK with the regtest fix
WORKDIR /build
RUN git clone https://github.com/Timi16/zingolib.git && \
    cd zingolib && \
    git checkout fix/regtest-chain-support && \
    rustup set profile minimal && \
    cargo build --release --package zingo-cli && \
    mv target/release/zingo-cli /usr/local/bin/ && \
    chmod +x /usr/local/bin/zingo-cli

# Create data directory
RUN mkdir -p /var/zingo && chmod 777 /var/zingo

WORKDIR /var/zingo

COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]