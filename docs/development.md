# Development

## Prerequisites

- Rust 1.88+
- Node.js 18+ (for the frontend and desktop app)
- Tauri v2 system dependencies (for desktop app only):
  - **macOS**: Xcode Command Line Tools (`xcode-select --install`)
  - **Linux**: `sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`
  - **Windows**: WebView2 (pre-installed on Windows 10/11)

## Getting started

```bash
# Clone
git clone git@github.com:n1nj4t4nuk1/collapse.git
cd collapse

# Build everything
cargo build

# Run the API (dev mode)
cargo run -p collapse-api

# In another terminal, start the frontend dev server
cd apps/web
npm install
npm run dev
```

The frontend dev server (Vite) runs on `http://localhost:5173` and proxies nothing by default -- it calls the API at `http://localhost:8000` (set via `VITE_API_URL`).

### Running the desktop app

```bash
cd apps/desktop
npm install
npx tauri dev
```

This starts Vite for the frontend and compiles the Tauri Rust backend with hot-reload.

## Running tests

```bash
# All tests (84 tests across core and api)
cargo test

# Single crate
cargo test -p collapse-core    # 34 tests
cargo test -p collapse-api     # 50 tests (unit + integration)
```

### Test structure

**collapse-core** -- unit tests in each module:
- `compression::tests` -- Algorithm enum, level validation, `from_extension()`, extract dispatcher (15 tests)
- `compression::zip::tests` -- ZIP compress & extract roundtrips (10 tests)
- `compression::sevenz::tests` -- 7z compress & extract roundtrips (10 tests, uses `sevenz_rust2::decompress` to verify)

**collapse-api** -- unit tests per module + integration tests:
- `models::tests` -- JobStatus serde, timestamps (5 tests)
- `registry::tests` -- CRUD operations on InMemoryJobRegistry (11 tests)
- `storage::tests` -- FilesystemStorage operations (12 tests)
- `api_tests` -- HTTP integration tests using `tower::ServiceExt::oneshot` (22 tests), including a full roundtrip: upload -> poll -> download -> verify ZIP content

Integration tests build the Axum router directly and send synthetic HTTP requests through it -- no TCP listener needed.

## Project structure cheat sheet

```
apps/core/src/
├── lib.rs                  # Re-exports compression module
└── compression.rs          # Algorithm enum, CompressionError, compress()/extract() dispatchers
    ├── zip.rs              # compress_zip(), extract_zip()
    └── sevenz.rs           # compress_7z(), extract_7z()

apps/api/src/
├── lib.rs                  # Module declarations + build_router() (public API)
├── main.rs                 # CLI entrypoint (binary)
├── config.rs               # Storage path helpers
├── models.rs               # JobStatus, CompressionJob
├── registry.rs             # InMemoryJobRegistry (Mutex<HashMap>)
├── storage.rs              # FilesystemStorage (disk I/O)
├── queue.rs                # Compression worker (mpsc consumer)
├── state.rs                # AppState (shared state for handlers)
├── routes.rs               # Axum handler functions (6 endpoints)
├── schemas.rs              # JSON response structs
├── error.rs                # AppError -> HTTP status mapping
└── api_tests.rs            # Integration tests

apps/aio/src/
└── main.rs                 # Same as api but always serves frontend

apps/cli/src/
└── main.rs                 # Subcommands: compress (c), extract (e)

apps/desktop/
├── src/                    # Vue 3 frontend
│   ├── main.js
│   └── App.vue             # Keka-style UI with drag-and-drop
├── src-tauri/              # Tauri v2 Rust backend
│   ├── Cargo.toml
│   ├── src/lib.rs          # compress_file, extract_file commands
│   ├── src/main.rs         # Entry point
│   └── tauri.conf.json     # Tauri configuration
├── package.json
├── vite.config.js
└── index.html

apps/web/src/
├── main.js                 # Vue app entry
├── App.vue                 # Root component (FileUpload + JobList)
├── services/api.js         # Fetch-based API client
├── composables/useJobs.js  # Reactive job list with polling
└── components/
    ├── FileUpload.vue      # Drag-and-drop upload form
    └── JobList.vue         # Job list with status, download, delete
```

## Adding a new compression algorithm

1. Add a variant to `Algorithm` in `apps/core/src/compression.rs` (with serde rename, `extension()`, `media_type()`, `Display`, `FromStr`, `from_extension()`).
2. Create `apps/core/src/compression/<name>.rs` with `compress_<name>()` and `extract_<name>()` functions.
3. Add `mod <name>;` to `apps/core/src/compression.rs` and new arms in the `compress()` and `extract()` match blocks.
4. Write tests in the new module.
5. No changes needed in the API, CLI, or desktop app -- they pick up new algorithms automatically through the `Algorithm` enum.

## Adding a new API endpoint

1. Add the handler function in `apps/api/src/routes.rs`.
2. If it needs a new response shape, add a struct to `apps/api/src/schemas.rs`.
3. Register the route in `build_router()` in `apps/api/src/lib.rs`.
4. Add integration tests in `apps/api/src/api_tests.rs`.
