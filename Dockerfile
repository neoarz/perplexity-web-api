# syntax=docker/dockerfile:1.7

FROM rust:1.93.1-slim-bookworm AS builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        clang \
        cmake \
        git \
        pkg-config \
        perl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock rustfmt.toml ./
COPY crates ./crates

RUN mkdir -p /out

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --locked --release -p perplexity-api-server \
    && cp /app/target/release/perplexity-api-server /out/perplexity-api-server

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        tzdata \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --shell /usr/sbin/nologin --uid 10001 appuser \
    && mkdir -p /app/logs \
    && chown -R appuser:appuser /app

WORKDIR /app

COPY --from=builder /out/perplexity-api-server /usr/local/bin/perplexity-api-server

ENV HOST=0.0.0.0
ENV PORT=3000
ENV RUST_LOG=info
ENV LOG_FILE=/app/logs/logs.txt

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl --fail --silent http://127.0.0.1:${PORT}/health >/dev/null || exit 1

USER appuser

ENTRYPOINT ["/usr/local/bin/perplexity-api-server"]
