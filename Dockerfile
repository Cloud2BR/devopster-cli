FROM rust:1.85-bookworm AS base

RUN rustup component add clippy rustfmt

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        git \
        make \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN cargo fetch

FROM base AS test
RUN cargo test

FROM base AS dev
CMD ["sleep", "infinity"]
