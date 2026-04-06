# Conventions & Rules

## Project Structure

```
app/
├── domain/           # Models, enums, port interfaces — NO external deps
│   ├── interface.py  # Interface(ABC) base class
│   ├── models/       # Dataclasses and enums
│   └── ports/        # Abstract interfaces (contracts)
├── application/      # Use-case orchestration — depends ONLY on domain
│   └── services/
├── infrastructure/   # Adapters, frameworks, I/O — implements ports
│   ├── api/          # HTTP layer (routes + schemas)
│   ├── persistence/  # Data storage adapters
│   ├── storage/      # File I/O adapters
│   └── compression/  # Compression algorithm adapters
├── container.py      # Composition root (DI wiring)
└── main.py           # FastAPI app + server startup
```

## Interface Rules

1. All interfaces inherit from `Interface(ABC)` (defined in `app/domain/interface.py`).
2. An interface **must only contain `@abstractmethod` methods**. No concrete methods, no attributes, no `__init__`.
3. `@property` combined with `@abstractmethod` is allowed.
4. Interfaces live in `app/domain/ports/`.

```python
from abc import abstractmethod
from app.domain.interface import Interface

class SomePort(Interface):
    @abstractmethod
    def do_something(self) -> None: ...

    @property
    @abstractmethod
    def name(self) -> str: ...
```

## Dependency Inversion

- Domain and application layers **never import** from `infrastructure/`.
- All external dependencies (py7zr, zipfile, filesystem, FastAPI) are wrapped behind a port.
- Concrete adapters are wired in `container.py` — the **only place** that knows about all implementations.

## Dependency Injection

- The `Container` class in `app/container.py` holds all wired dependencies.
- A module-level instance `container` is the singleton used at runtime.
- Consumers access it via `import app.container as _di` and use `_di.container.xxx`.
  - **Do not** use `from app.container import container` (captures a fixed reference that breaks test isolation).
- The `Container` constructor accepts optional overrides for every port, making it trivial to swap adapters in tests.

## Naming

| Kind | Convention | Example |
|---|---|---|
| Port (interface) | `<Name>Port` | `JobRegistryPort`, `FileStoragePort` |
| Adapter (implementation) | Descriptive class name | `InMemoryJobRegistry`, `FilesystemStorage` |
| Compression adapter | `<Format>Compression` | `SevenZipCompression`, `ZipCompression` |
| Schema (Pydantic) | `<Purpose>Response` | `UploadAcceptedResponse`, `DeleteResponse` |
| Test file | `test_<module>.py` | `test_api.py`, `test_registry.py` |

## Adding a New Compression Algorithm

1. Create a class in `app/infrastructure/compression/` implementing `CompressionStrategyPort`.
2. Add the algorithm value to `CompressionAlgorithm` enum in `app/domain/models/job.py`.
3. Register it in the `Container.__init__` strategies dict in `app/container.py`.

No other files need to change — the route, service, and queue are algorithm-agnostic.

## Adding a New Persistence Backend

1. Create a class in `app/infrastructure/persistence/` implementing `JobRegistryPort`.
2. Swap it in `Container.__init__` (or add a constructor parameter/env-var switch).

## API Route Rules

- Routes are **thin adapters**: translate HTTP → call domain/application objects → translate back to HTTP.
- Routes access dependencies via `_di.container.<dependency>`.
- Framework-specific types (`UploadFile`, `HTTPException`) stay in the route layer.
- Response schemas live in `infrastructure/api/schemas/`.

## Testing

- Every test gets a fresh `Container` via the `fresh_container` autouse fixture (see `tests/conftest.py`).
- Unit tests create adapters directly — no container needed.
- API integration tests use the `client` fixture (FastAPI `TestClient`), which mocks the compression queue start/stop to avoid event-loop conflicts.
- When testing uploads, mock `file_storage.save_file` and `compression_queue.enqueue` to avoid real I/O.

## Configuration

- Storage paths and chunk size are in `app/infrastructure/config.py`.
- Host/port resolution: CLI args > env vars (`COLLAPSE_HOST`/`COLLAPSE_PORT`) > defaults (`0.0.0.0:8000`).
- `FilesystemStorage` receives paths via constructor (not global imports) for testability.

## General

- Python 3.11+ required.
- Single-process only — the in-memory registry is not shared across workers.
- No database — job state is ephemeral by design.
- Compression runs in `asyncio.to_thread()` to avoid blocking the event loop.
