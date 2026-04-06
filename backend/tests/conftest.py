"""Shared pytest fixtures for the Collapse test suite."""

from unittest.mock import AsyncMock, patch

import pytest
from fastapi.testclient import TestClient

import app.container as container_mod
from app.container import Container
from app.infrastructure.persistence.in_memory_job_registry import InMemoryJobRegistry
from app.main import app


@pytest.fixture(autouse=True)
def fresh_container():
    """Replace the global container with one backed by a fresh in-memory registry."""
    old = container_mod.container
    container_mod.container = Container(job_registry=InMemoryJobRegistry())
    yield container_mod.container
    container_mod.container = old


@pytest.fixture()
def fresh_registry(fresh_container):
    """Shortcut to access the registry from the current test container."""
    return fresh_container.job_registry


@pytest.fixture()
def client(fresh_container):
    """Return a synchronous TestClient for the FastAPI app.

    The compression queue worker start/stop are mocked so tests do not create
    asyncio tasks that would be bound to the wrong event loop across test runs.
    """
    with (
        patch.object(fresh_container.compression_queue, "start", new_callable=AsyncMock),
        patch.object(fresh_container.compression_queue, "stop", new_callable=AsyncMock),
    ):
        with TestClient(app) as c:
            yield c
