FROM rust:1.88-slim AS builder

WORKDIR /src
COPY Cargo.toml ./
COPY apps/core apps/core
COPY apps/api apps/api
COPY apps/cli apps/cli

RUN cargo build --release -p collapse-api

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /src/target/release/collapse-api /usr/local/bin/collapse-api

EXPOSE 8000

CMD ["collapse-api"]
