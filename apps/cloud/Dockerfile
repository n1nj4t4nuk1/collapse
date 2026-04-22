FROM rust:1.86-slim AS builder

WORKDIR /src
COPY Cargo.toml ./
COPY apps/core apps/core
COPY apps/cloud apps/cloud
COPY apps/cli apps/cli

RUN cargo build --release -p collapse-cloud

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /src/target/release/collapse-cloud /usr/local/bin/collapse-cloud

EXPOSE 8000

CMD ["collapse-cloud"]
