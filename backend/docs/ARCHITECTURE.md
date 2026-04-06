# Architecture

Collapse follows **hexagonal architecture** (ports & adapters) with strict dependency inversion.

## Layers

```
┌──────────────────────────────────────────────────────────┐
│                    infrastructure/                        │
│  ┌─────────────┐  ┌────────────┐  ┌──────────────────┐  │
│  │  API (HTTP)  │  │ Filesystem │  │  Compression     │  │
│  │  routes +    │  │  Storage   │  │  (py7zr, zipfile)│  │
│  │  schemas     │  │            │  │                  │  │
│  └──────┬───────┘  └─────┬──────┘  └───────┬──────────┘  │
│         │                │                  │             │
│  ┌──────┴────────────────┴──────────────────┴──────────┐  │
│  │                  container.py                       │  │
│  │              (composition root / DI wiring)         │  │
│  └──────┬────────────────┬──────────────────┬──────────┘  │
└─────────┼────────────────┼──────────────────┼─────────────┘
          │                │                  │
┌─────────┼────────────────┼──────────────────┼─────────────┐
│         ▼                ▼                  ▼  application │
│   CompressionService          CompressionQueueService     │
│   (orchestrates jobs)         (async FIFO worker)         │
└─────────┬────────────────┬──────────────────┬─────────────┘
          │                │                  │
┌─────────┼────────────────┼──────────────────┼─────────────┐
│         ▼                ▼                  ▼    domain    │
│   JobRegistryPort   FileStoragePort   CompressionStrategy │
│                                       Port                │
│                  CompressionJob model                      │
│              JobStatus, CompressionAlgorithm               │
└───────────────────────────────────────────────────────────┘
```

### Domain (`app/domain/`)

Pure business logic. **Zero external dependencies** — only Python stdlib.

- **`interface.py`** — `Interface(ABC)` base class. Every subclass is a pure interface: all methods must be `@abstractmethod`.
- **`models/job.py`** — `CompressionJob` dataclass, `JobStatus` and `CompressionAlgorithm` enums.
- **`ports/`** — Interfaces that define the contracts the outside world must satisfy:
  - `JobRegistryPort` — store, query, update, remove jobs.
  - `FileStoragePort` — save/delete files, generate paths. Accepts `BinaryIO` (not FastAPI's `UploadFile`).
  - `CompressionStrategyPort` — compress a file given source path, destination, filename, and level.

### Application (`app/application/`)

Orchestration logic. Depends **only on domain** (ports + models).

- **`compression_service.py`** — Drives the `QUEUED → COMPRESSING → COMPLETED | FAILED` state machine. Offloads blocking compression to `asyncio.to_thread()`.
- **`compression_queue.py`** — Async FIFO queue with a single worker task. Receives a `Callable[[str], Awaitable[None]]` (no direct dependency on `CompressionService`).

### Infrastructure (`app/infrastructure/`)

Concrete implementations of ports, plus framework-specific code.

| Subdirectory | Contents |
|---|---|
| `persistence/` | `InMemoryJobRegistry` — dict + `threading.Lock` |
| `storage/` | `FilesystemStorage` — local disk I/O, receives `input_dir`/`output_dir` via constructor |
| `compression/` | `SevenZipCompression` (py7zr), `ZipCompression` (stdlib zipfile) |
| `api/routes/` | FastAPI router — thin HTTP adapter |
| `api/schemas/` | Pydantic response models |
| `config.py` | Path constants (`STORAGE_DIR`, `INPUT_DIR`, `OUTPUT_DIR`, `MAX_UPLOAD_CHUNK_SIZE`) |

### Composition Root (`app/container.py`)

Wires everything together. The `Container` class accepts optional overrides for testing:

```python
Container(
    job_registry=InMemoryJobRegistry(),          # or any JobRegistryPort
    file_storage=FilesystemStorage(...),          # or any FileStoragePort
    compression_strategies={...},                 # or any strategy map
)
```

All consumers access the container through the **module attribute** `app.container.container` (via `import app.container as _di`), which allows test fixtures to swap it at runtime.

## Dependency Rule

Dependencies point **inward only**:

```
infrastructure  →  application  →  domain
```

- Domain knows nothing about FastAPI, py7zr, or the filesystem.
- Application knows nothing about HTTP, specific compression libraries, or storage backends.
- Infrastructure implements the ports and plugs into the container.

## Key Design Decisions

| Decision | Rationale |
|---|---|
| Single-process, in-memory registry | No database needed; restart clears state (by design) |
| `asyncio.to_thread()` for compression | CPU-bound work must not block the event loop |
| Single queue worker | Sequential processing, simple; no concurrency bugs |
| `BinaryIO` on storage port (not `UploadFile`) | Decouples domain from FastAPI |
| `Callable` on queue (not `CompressionService`) | Avoids circular dependency between queue and service |
| `threading.Lock` on registry | HTTP handlers and compression worker share state across threads |
