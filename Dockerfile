# syntax=docker/dockerfile:1.7

FROM rust:1.86-slim-bookworm AS builder

WORKDIR /workspace

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/workspace/target \
    cargo build --locked --release -p bitwarden-eso-provider \
    && cp /workspace/target/release/bitwarden-eso-provider /usr/local/bin/bitwarden-eso-provider

FROM debian:bookworm-slim AS runtime

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 65532 --home-dir /nonexistent --shell /usr/sbin/nologin bweso

COPY --from=builder /usr/local/bin/bitwarden-eso-provider /usr/local/bin/bitwarden-eso-provider

USER 65532:65532
EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/bitwarden-eso-provider"]
