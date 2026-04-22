# Architecture

Collapse is a Rust workspace monorepo for file compression. It accepts uploads, compresses them in the background, and lets clients download the result.

## Project layout

```
collapse/
├── Cargo.toml              # Workspace root
├── Makefile                 # Build & Docker targets
├── apps/
│   ├── core/               # collapse-core   (lib)
│   ├── api/                # collapse-api    (lib + bin)
│   ├── aio/                # collapse-aio    (bin)
│   ├── cli/                # collapse-cli    (bin)
│   ��── web/                # Vue 3 SPA (not a Rust crate)
├── docs/
└── .github/workflows/
```

## Crate dependency graph

```
collapse-core          (no internal deps)
    ^
    |
collapse-api           (depends on core)
    ^
    |
collapse-aio           (depends on api, which re-exports core)

collapse-cli           (depends on core)
```

## Crate responsibilities

### collapse-core (`apps/core`)

Pure library, no I/O beyond reading/writing files. Contains:

- **`Algorithm` enum** -- `SevenZ` / `Zip`, with serde rename, `Display`, `FromStr`, extension and MIME type helpers.
- **`CompressionError`** -- `Io`, `Failed`, `InvalidLevel` via `thiserror`.
- **`compress(source, output, arcname, algorithm, level)`** -- dispatcher that validates the level (1--5) and delegates to the algorithm-specific function.
- **`compress_7z`** -- wraps `sevenz-rust2` (`SevenZWriter`).
- **`compress_zip`** -- wraps `zip` crate (`ZipWriter` + Deflate).

API levels 1--5 are mapped to internal presets `[1, 3, 5, 7, 9]` for both algorithms.

### collapse-api (`apps/api`)

HTTP backend. Structured as **lib + bin** so other crates (aio) can import the router.

**Library (`lib.rs`) exports:**

| Module | Key types / functions |
|--------|-----------------------|
| `config` | `input_dir()`, `output_dir()` -- storage paths |
| `models` | `JobStatus` enum (Queued/Compressing/Completed/Failed), `CompressionJob` struct |
| `registry` | `InMemoryJobRegistry` -- `Mutex<HashMap>` job store |
| `storage` | `FilesystemStorage` -- disk I/O (save, delete, path builders) |
| `queue` | `start_compression_worker(registry, rx)` -- spawns tokio task consuming from an mpsc channel |
| `state` | `AppState` (Clone) -- `Arc<Registry>` + `Arc<Storage>` + `mpsc::UnboundedSender` |
| `build_router(state)` | Returns an `axum::Router` with all API routes + CORS |

**Private modules** (internal to the lib, not exported):

| Module | Purpose |
|--------|---------|
| `routes` | 6 Axum handler functions |
| `schemas` | Serializable response structs |
| `error` | `AppError` enum implementing `IntoResponse` |

**Binary (`main.rs`):** CLI entrypoint using `clap`. Parses `--host`/`--port`, sets up state, builds router, optionally serves a `static/` directory, and starts the Axum server.

### collapse-aio (`apps/aio`)

All-in-one server. Identical to the API binary but **always** serves the compiled Vue frontend as a fallback route. This is the recommended deployment mode when you want a single container for everything.

The static directory defaults to `static/` and is overridable via `COLLAPSE_STATIC_DIR`.

### collapse-cli (`apps/cli`)

Placeholder for a command-line compression tool. Depends on `collapse-core`.

### Frontend (`apps/web`)

Vue 3 SPA built with Vite. Communicates with the API via `fetch`. Key files:

| File | Purpose |
|------|---------|
| `src/services/api.js` | API client -- all 6 endpoints, uses `VITE_API_URL` for base URL |
| `src/composables/useJobs.js` | Polls `GET /files` every 2 seconds for live status updates |
| `src/components/FileUpload.vue` | Drag-and-drop upload with algorithm/level selectors |
| `src/components/JobList.vue` | Renders job list with download/delete actions |

## Request flow

```
Client ──POST /files──> upload_file handler
                            │
                            ├─ save file to disk (spawn_blocking)
                            ├─ create CompressionJob (status: Queued)
                            ├─ add to InMemoryJobRegistry
                            ├─ send job_id to mpsc channel
                            └─ return 202 Accepted

mpsc channel ──> compression worker (tokio task)
                            │
                            ├─ set status: Compressing
                            ├─ call collapse_core::compress (spawn_blocking)
                            └─ set status: Completed / Failed

Client ──GET /files/{id}/status──> poll until completed

Client ──GET /files/{id}/download──> download_archive handler
                            │
                            └─ read compressed file, return with Content-Disposition
```

## Concurrency model

- **Axum** runs on the tokio multi-threaded runtime.
- **CPU-bound compression** is offloaded to the blocking thread pool via `tokio::task::spawn_blocking`, so it never blocks the async executor.
- **Job registry** uses `std::sync::Mutex` (not `tokio::Mutex`) because the lock is never held across `.await` points.
- **Compression queue** uses a `tokio::sync::mpsc::unbounded_channel`. Jobs are processed sequentially by a single worker task. This mirrors the original Python implementation and guarantees predictable disk I/O pressure.

## State management

All state is in-memory. There is no database. Restarting the process loses all jobs.

- `InMemoryJobRegistry`: `Mutex<HashMap<String, CompressionJob>>` -- the single source of truth.
- `FilesystemStorage`: stores uploaded files in `storage/input/`, compressed archives in `storage/output/`.
- `AppState` bundles both plus the queue sender. It implements `Clone` (all fields are `Arc`) and is injected into every handler via Axum's `State` extractor.

## Error handling

`AppError` maps application errors to HTTP status codes:

| Variant | HTTP status | When |
|---------|-------------|------|
| `NotFound` | 404 | Job ID doesn't exist, archive file missing |
| `Conflict` | 409 | Download/delete while compressing or queued |
| `BadRequest` | 400 | Missing file, empty filename, invalid level |
| `Internal` | 500 | I/O failures, task panics |

All error responses are JSON: `{ "detail": "..." }`.
