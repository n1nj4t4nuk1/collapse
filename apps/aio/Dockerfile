FROM node:22-slim AS frontend

WORKDIR /app
COPY apps/web/package.json apps/web/package-lock.json ./
RUN npm ci
COPY apps/web/ ./
ENV VITE_API_URL=""
RUN npm run build

FROM rust:1.88-slim AS builder

WORKDIR /src
COPY Cargo.toml ./
COPY apps/core apps/core
COPY apps/api apps/api
COPY apps/aio apps/aio
COPY apps/cli apps/cli
COPY apps/desktop/src-tauri apps/desktop/src-tauri

RUN cargo build --release -p collapse-aio

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /src/target/release/collapse-aio /usr/local/bin/collapse-aio
COPY --from=frontend /app/dist ./static

EXPOSE 8000

CMD ["collapse-aio"]
