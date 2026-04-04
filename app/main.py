"""
Main entry point for the Collapse application.

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

from fastapi import FastAPI
import uvicorn

from app.api.routes.files import router as files_router
from app.services.compression_queue import compression_queue_service
from app.services.storage import storage_service

DEFAULT_HOST = "0.0.0.0"
DEFAULT_PORT = 8000


@asynccontextmanager
async def lifespan(_: FastAPI):
    """
    Application lifespan context manager.

    On startup: ensures storage directories exist and starts the compression
    queue worker. On shutdown: stops the worker gracefully.
    """
    storage_service.ensure_directories()
    await compression_queue_service.start()
    yield
    await compression_queue_service.stop()


app = FastAPI(
    title="Collapse",
    description="A simple file compression API.",
    version="0.1.0",
    lifespan=lifespan,
)
app.include_router(files_router)


def _parse_args() -> argparse.Namespace:
    """Parse CLI arguments for host and port."""
    parser = argparse.ArgumentParser(description="Collapse – file compression API server.")
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
    """
    Start the Uvicorn server.

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
