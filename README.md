# Collapse

Collapse is a file compression web service that accepts file uploads, compresses them using configurable algorithms, and serves the results through a REST API and a Vue 3 frontend.

## Architecture

The project is a Rust workspace organized as a monorepo under `apps/`:

| Crate | Path | Description |
|-------|------|-------------|
| `collapse-core` | `apps/core` | Shared compression library (7z, ZIP) |
| `collapse-api` | `apps/api` | HTTP backend built with Axum |
| `collapse-aio` | `apps/aio` | All-in-one server (API + frontend) |
| `collapse-cli` | `apps/cli` | CLI tool for local compression |
| Frontend | `apps/web` | Vue 3 SPA |

## Features

- Upload one file per request.
- Choose between **7z** (LZMA2) and **zip** (Deflate) compression.
- Choose a compression level from **1** (fastest) to **5** (maximum).
- Track jobs entirely in memory -- no database required.
- Download the generated archive.
- Delete individual jobs or bulk-delete all completed jobs.
- Vue 3 frontend with real-time job status tracking.

## CI

GitHub Actions pipeline (`test_and_build.yml`) runs on every push to `main` and on PRs:

1. **Test** -- runs `cargo test` for the entire workspace.
2. **Build** -- debug build per app in parallel (`collapse-core`, `collapse-api`, `collapse-aio`, `collapse-cli`).
3. **Release build** -- `cargo build --release` per app in parallel.

Docker images are validated separately via `docker-build.yml`.

## Requirements

- Rust 1.88+ (2021 edition)
- Node.js 18+ (for building the frontend)

## Build

```bash
# Build individual apps
make api
make web
make cli

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
cargo run -p collapse-cli
```

Host and port can be set via `--host`/`--port` flags, `COLLAPSE_HOST`/`COLLAPSE_PORT` env vars, or defaults to `0.0.0.0:8000`.

## Tests

```bash
# Run all tests (65 tests across core and api)
cargo test

# Run tests for a specific crate
cargo test -p collapse-core
cargo test -p collapse-api
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

## Notes

- Job state lives only in memory. Restarting the application clears the registry.
- The service runs as a single-process app because multiple workers would not share the in-memory job registry.
- Compression runs on a dedicated background worker via a `tokio::sync::mpsc` channel, keeping the HTTP handlers non-blocking.
