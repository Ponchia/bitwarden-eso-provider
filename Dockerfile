FROM rust:1.86-slim-bookworm AS builder

WORKDIR /workspace

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo build --release -p bitwarden-eso-provider

FROM debian:bookworm-slim AS runtime

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 65532 --home-dir /nonexistent --shell /usr/sbin/nologin bweso

COPY --from=builder /workspace/target/release/bitwarden-eso-provider /usr/local/bin/bitwarden-eso-provider

USER 65532:65532
EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/bitwarden-eso-provider"]
