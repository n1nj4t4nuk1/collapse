# Deployment

## Makefile targets

All commands run from the repository root.

### Build targets

| Target | Description |
|--------|-------------|
| `make api` | `cargo build --release -p collapse-api` |
| `make cli` | `cargo build --release -p collapse-cli` |
| `make web` | `cd apps/web && npm ci && npm run build` |
| `make aio` | Builds frontend first, then `cargo build --release -p collapse-aio` |
| `make desktop` | `cd apps/desktop && npm ci && npm run build` then `cargo build --release -p collapse-desktop` |

### Docker targets

| Target | Description |
|--------|-------------|
| `make api/docker/build` | Build `collapse-api` image (API only, no frontend) |
| `make web/docker/build` | Build `collapse-web` image (nginx serving the SPA) |
| `make aio/docker/build` | Build `collapse-aio` image (API + frontend in one container) |

## Docker images

### collapse-api

Multi-stage build:
1. `rust:1.88-slim` -- compiles the release binary.
2. `debian:bookworm-slim` -- copies only the binary. Minimal image.

```
docker run -p 8000:8000 collapse-api
```

API only. No frontend. Useful when the frontend is deployed separately (e.g., behind a CDN or in the `collapse-web` container).

### collapse-web

Multi-stage build:
1. `node:22-slim` -- runs `npm ci && npm run build`.
2. `nginx:alpine` -- serves the built SPA with a custom `nginx.conf` that handles SPA routing (`try_files $uri $uri/ /index.html`).

```
docker run -p 80:80 collapse-web
```

The frontend expects the API at the URL defined by `VITE_API_URL` at build time (defaults to empty string = same origin).

### collapse-aio

Multi-stage build:
1. `node:22-slim` -- builds the frontend.
2. `rust:1.88-slim` -- compiles the Rust binary.
3. `debian:bookworm-slim` -- copies the binary and the frontend dist into `/app/static`.

```
docker run -p 8000:8000 collapse-aio
```

Single container serving both the API and the frontend. The binary runs with `WORKDIR /app` and finds the static files at `./static/`.

## Environment variables

| Variable | Default | Used by | Description |
|----------|---------|---------|-------------|
| `COLLAPSE_HOST` | `0.0.0.0` | api, aio | Bind address |
| `COLLAPSE_PORT` | `8000` | api, aio | Listen port |
| `COLLAPSE_STATIC_DIR` | `static` | aio | Path to the compiled frontend |
| `VITE_API_URL` | `""` (same origin) | web | API base URL, set at frontend build time |

CLI flags `--host` and `--port` take precedence over env vars.

## CI pipelines

### test_and_build.yml

Triggers on push to `main`/`master` and on PRs.

```
test ──> build (matrix: 5 apps) ──> release-build (matrix: 5 apps)
```

1. **test** -- `cargo test` for the entire workspace.
2. **build** -- `cargo build -p <package>` per app in parallel (core, api, aio, cli, desktop).
3. **release-build** -- `cargo build --release -p <package>` per app in parallel.

All stages install Tauri system dependencies (`libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `patchelf`) for the desktop build.

### docker-build.yml

Triggers on push to `main`/`master` and on PRs. Builds all three Docker images in parallel (api, web, aio) without pushing -- validates that the Dockerfiles are correct.

### mirror-codeberg.yml

Mirrors the repository to Codeberg on every push.

## Storage

Files are stored on the local filesystem:

```
storage/
├── input/    # Uploaded original files (UUID-based names)
└── output/   # Compressed archives ({job_id}.{ext})
```

In Docker, this directory is created inside the container at runtime. For persistence across restarts, mount a volume:

```
docker run -v /data/collapse:/app/storage -p 8000:8000 collapse-aio
```
