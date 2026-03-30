FROM mcr.microsoft.com/devcontainers/rust:1-1-bookworm

RUN rustup component add clippy rustfmt

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        git \
        make \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspaces/devopster-cli

COPY . .

RUN cargo fetch

CMD ["sleep", "infinity"]
