"""Main entry point for the Collapse application.

Initialises the FastAPI app, registers routers, and manages the server
lifecycle: storage directory setup and compression queue worker startup/shutdown.

Server host and port can be configured through (highest priority first):
  1. CLI arguments  --host / --port
  2. Environment variables  COLLAPSE_HOST / COLLAPSE_PORT
  3. Built-in defaults  0.0.0.0 / 8000
"""

import argparse
import os
from contextlib import asynccontextmanager
from pathlib import Path

import uvicorn
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import FileResponse
from fastapi.staticfiles import StaticFiles

import app.container as _di
from app.infrastructure.api.routes.files import router as files_router

STATIC_DIR = Path(__file__).resolve().parent.parent / "static"

DEFAULT_HOST = "0.0.0.0"
DEFAULT_PORT = 8000


@asynccontextmanager
async def lifespan(_: FastAPI):
    """Application lifespan context manager.

    On startup: ensures storage directories exist and starts the compression
    queue worker. On shutdown: stops the worker gracefully.
    """
    _di.container.file_storage.ensure_directories()
    await _di.container.compression_queue.start()
    yield
    await _di.container.compression_queue.stop()


app = FastAPI(
    title="Collapse",
    description="A simple file compression API.",
    version="0.1.0",
    lifespan=lifespan,
)
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)
app.include_router(files_router)

if STATIC_DIR.is_dir():
    @app.get("/{full_path:path}")
    async def serve_frontend(full_path: str):
        """Serve the Vue SPA. Static assets are served directly; all other
        paths return index.html for client-side routing."""
        file = STATIC_DIR / full_path
        if file.is_file():
            return FileResponse(file)
        return FileResponse(STATIC_DIR / "index.html")


def _parse_args() -> argparse.Namespace:
    """Parse CLI arguments for host and port."""
    parser = argparse.ArgumentParser(description="Collapse \u2013 file compression API server.")
    parser.add_argument(
        "--host",
        default=None,
        help="Host address to bind to (overrides COLLAPSE_HOST env var, default: %(const)s).",
    )
    parser.add_argument(
        "--port",
        type=int,
        default=None,
        help="Port to listen on (overrides COLLAPSE_PORT env var, default: %(const)s).",
    )
    return parser.parse_args()


def run() -> None:
    """Start the Uvicorn server.

    Resolution order for host and port:
      CLI argument > environment variable (COLLAPSE_HOST / COLLAPSE_PORT) > built-in default.
    """
    args = _parse_args()

    host = args.host or os.environ.get("COLLAPSE_HOST") or DEFAULT_HOST
    _env_port = os.environ.get("COLLAPSE_PORT")
    port = args.port or (int(_env_port) if _env_port else DEFAULT_PORT)

    uvicorn.run("app.main:app", host=host, port=port, reload=False)


if __name__ == "__main__":
    run()
