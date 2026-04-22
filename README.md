# Collapse

Collapse is a file compression toolkit that supports compressing and extracting files using 7z (LZMA2) and ZIP (Deflate) algorithms. It ships as a REST API, a web frontend, a CLI tool, and a cross-platform desktop app.

## Architecture

The project is a Rust workspace organized as a monorepo under `apps/`:

| Crate | Path | Description |
|-------|------|-------------|
| `collapse-core` | `apps/core` | Shared compression & extraction library (7z, ZIP) |
| `collapse-api` | `apps/api` | HTTP backend built with Axum |
| `collapse-aio` | `apps/aio` | All-in-one server (API + frontend) |
| `collapse-cli` | `apps/cli` | CLI tool for compression & extraction |
| `collapse-desktop` | `apps/desktop` | Cross-platform desktop app (Tauri v2) |
| Frontend | `apps/web` | Vue 3 SPA |

## Features

- **Compress** files using **7z** (LZMA2) or **ZIP** (Deflate) with levels 1--5.
- **Extract** archives (auto-detects format by extension).
- REST API with background job processing (in-memory, no database).
- Vue 3 frontend with real-time job status tracking.
- CLI with `compress`/`extract` subcommands.
- Desktop app (Tauri v2) with drag-and-drop, Keka-style UI (macOS, Windows, Linux).

## CI

GitHub Actions pipeline (`test_and_build.yml`) runs on every push to `main` and on PRs:

1. **Test** -- runs `cargo test` for the entire workspace.
2. **Build** -- debug build per app in parallel (`collapse-core`, `collapse-api`, `collapse-aio`, `collapse-cli`, `collapse-desktop`).
3. **Release build** -- `cargo build --release` per app in parallel.

Docker images are validated separately via `docker-build.yml`.

## Requirements

- Rust 1.88+ (2021 edition)
- Node.js 18+ (for building the frontend and desktop app)
- Tauri v2 system dependencies (for desktop app -- see [development docs](docs/development.md))

## Build

```bash
# Build individual apps
make api
make web
make cli
make desktop

# Build AIO (frontend + backend)
make aio

# Docker images
make api/docker/build
make web/docker/build
make aio/docker/build
```

## Run

### API (HTTP server only)

```bash
cargo run -p collapse-api -- --host 0.0.0.0 --port 8000
```

### AIO (API + frontend)

```bash
cargo run -p collapse-aio -- --host 0.0.0.0 --port 8000
```

Serves the Vue frontend at `/` and the API at `/files`. Set `COLLAPSE_STATIC_DIR` to override the static files path (defaults to `static/`).

### CLI

```bash
# Compress
collapse compress archivo.txt --protocol zip --level 3

# Extract
collapse extract archivo.zip --output ./destino

# Short aliases
collapse c archivo.txt
collapse e archivo.7z
```

### Desktop

```bash
# Development mode (hot-reload)
cd apps/desktop && npx tauri dev

# Release build
make desktop
```

## Tests

```bash
# Run all tests (84 tests across core and api)
cargo test

# Run tests for a specific crate
cargo test -p collapse-core    # 34 tests
cargo test -p collapse-api     # 50 tests
```

## API Endpoints

### `POST /files`

Upload a file and start a compression job.

**Form fields:**

| Field       | Type    | Default | Description                                      |
|-------------|---------|---------|--------------------------------------------------|
| `file`      | file    | --       | The file to upload (required).                   |
| `algorithm` | string  | `7z`    | Compression algorithm: `7z` or `zip`.            |
| `level`     | integer | `5`     | Compression level: `1` (fastest) to `5` (max).  |

**Compression level mapping:**

| Level | 7z preset | ZIP compresslevel |
|-------|-----------|-------------------|
| 1     | 1         | 1                 |
| 2     | 3         | 3                 |
| 3     | 5         | 5                 |
| 4     | 7         | 7                 |
| 5     | 9         | 9                 |

**Response:** `202 Accepted` -- returns `job_id`, `filename`, `status`, `algorithm`, and `level`.

---

### `GET /files`

Lists all compression jobs.

---

### `GET /files/{job_id}/status`

Returns the current status of a compression job, including `algorithm`, `level`, and `archive_name`.

---

### `GET /files/{job_id}/download`

Downloads the compressed archive (`.7z` or `.zip`) once the job status is `completed`.

---

### `DELETE /files/{job_id}`

Deletes the original uploaded file and the compressed archive. Cannot be called while compression is in progress.

---

### `DELETE /files/completed`

Deletes all jobs with status `completed`, removes their original and compressed files from storage, and returns a summary with `deleted_jobs` and `deleted_files`.

## Documentation

See the [`docs/`](docs/) folder for detailed documentation:

- [Architecture](docs/architecture.md) -- crate responsibilities, dependency graph, request flow
- [API Reference](docs/api.md) -- all HTTP endpoints with request/response examples
- [Deployment](docs/deployment.md) -- Docker images, Makefile targets, environment variables, CI
- [Development](docs/development.md) -- getting started, test structure, adding algorithms

## Notes

- Job state lives only in memory. Restarting the application clears the registry.
- The service runs as a single-process app because multiple workers would not share the in-memory job registry.
- Compression runs on a dedicated background worker via a `tokio::sync::mpsc` channel, keeping the HTTP handlers non-blocking.
