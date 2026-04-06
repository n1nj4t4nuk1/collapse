# LLM Context — Quick Project Understanding

Read this file first to understand the project without exploring every file.

## What Is This?

**Collapse** is a FastAPI REST API that accepts file uploads, compresses them in the background (7z or ZIP), and lets clients download the result. All state lives in memory — no database.

## Tech Stack

- Python 3.11+, FastAPI, Uvicorn, py7zr (7z compression), zipfile (stdlib)

## Directory Map

```
app/
├── domain/                              # PURE BUSINESS LOGIC (no external deps)
│   ├── interface.py                     # Interface(ABC) — base for all interfaces
│   ├── models/job.py                    # CompressionJob dataclass + enums
│   └── ports/                           # Contracts (abstract interfaces)
│       ├── job_registry.py              # JobRegistryPort — CRUD for jobs
│       ├── file_storage.py              # FileStoragePort — file I/O
│       └── compression_strategy.py      # CompressionStrategyPort — compress a file
│
├── application/                         # ORCHESTRATION (depends only on domain)
│   └── services/
│       ├── compression_service.py       # Runs compression: QUEUED→COMPRESSING→COMPLETED|FAILED
│       └── compression_queue.py         # Async FIFO queue + single worker task
│
├── infrastructure/                      # ADAPTERS (implements ports, uses frameworks)
│   ├── config.py                        # Paths: storage/input, storage/output
│   ├── persistence/
│   │   └── in_memory_job_registry.py    # Dict + Lock → JobRegistryPort
│   ├── storage/
│   │   └── filesystem_storage.py        # Local disk → FileStoragePort
│   ├── compression/
│   │   ├── sevenz_strategy.py           # py7zr → CompressionStrategyPort
│   │   └── zip_strategy.py             # zipfile → CompressionStrategyPort
│   └── api/
│       ├── routes/files.py              # All HTTP endpoints (POST, GET, DELETE)
│       └── schemas/                     # Pydantic response models
│
├── container.py                         # DI WIRING — binds ports to adapters
└── main.py                              # FastAPI app, lifespan, CLI args
```

## How Dependencies Flow

```
infrastructure → application → domain
     (outer)       (middle)     (inner)
```

Nothing in domain or application imports from infrastructure. All external libraries (py7zr, FastAPI, filesystem) are behind port interfaces.

## Ports ↔ Adapters Mapping

| Port (interface) | Adapter (implementation) | External Dep |
|---|---|---|
| `JobRegistryPort` | `InMemoryJobRegistry` | None (dict + Lock) |
| `FileStoragePort` | `FilesystemStorage` | Filesystem |
| `CompressionStrategyPort` | `SevenZipCompression` | py7zr |
| `CompressionStrategyPort` | `ZipCompression` | zipfile (stdlib) |

## Container (DI)

`app/container.py` is the composition root. It wires ports → adapters and builds services:

```python
container.job_registry          # JobRegistryPort → InMemoryJobRegistry
container.file_storage          # FileStoragePort → FilesystemStorage
container.compression_service   # CompressionService(registry, strategies)
container.compression_queue     # CompressionQueueService(compress callback)
```

Access pattern: `import app.container as _di`, then `_di.container.xxx`. Never use `from app.container import container` (breaks test isolation).

## API Endpoints (all under /files)

| Method | Path | What It Does |
|---|---|---|
| `POST` | `/files` | Upload file → create job → enqueue compression |
| `GET` | `/files` | List all jobs |
| `GET` | `/files/{job_id}/status` | Job status |
| `GET` | `/files/{job_id}/download` | Download compressed archive |
| `DELETE` | `/files/{job_id}` | Delete job + files |
| `DELETE` | `/files/completed` | Bulk-delete all completed jobs |

## Job Lifecycle

`QUEUED → COMPRESSING → COMPLETED | FAILED`

## Interface Rule

All interfaces inherit from `Interface(ABC)`. An interface **must only have `@abstractmethod` methods** — no concrete implementations allowed.

## Testing

Tests replace the global container with a fresh one per test (`conftest.py::fresh_container` autouse fixture). Unit tests create adapters directly. API tests use `TestClient` with mocked queue start/stop.

## How to Add a New Compression Algorithm

1. Add value to `CompressionAlgorithm` enum (`domain/models/job.py`)
2. Create adapter in `infrastructure/compression/` implementing `CompressionStrategyPort`
3. Register it in `Container.__init__` (`container.py`)

## How to Add a New Storage Backend

1. Create adapter implementing `FileStoragePort`
2. Swap it in `Container.__init__`

## How to Add a New Persistence Backend

1. Create adapter implementing `JobRegistryPort`
2. Swap it in `Container.__init__`

## Configuration

| Setting | Source | Default |
|---|---|---|
| Host | `--host` CLI / `COLLAPSE_HOST` env | `0.0.0.0` |
| Port | `--port` CLI / `COLLAPSE_PORT` env | `8000` |
| Storage | `app/infrastructure/config.py` | `./storage/input`, `./storage/output` |

## Constraints

- Single-process only (in-memory registry, not distributed)
- No persistence across restarts (by design)
- Compression is CPU-bound → offloaded to `asyncio.to_thread()`
