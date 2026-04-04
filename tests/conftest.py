"""Shared pytest fixtures for the Collapse test suite."""

from unittest.mock import AsyncMock, patch

import pytest
from fastapi.testclient import TestClient

from app.main import app
from app.state.in_memory_registry import InMemoryJobRegistry


@pytest.fixture(autouse=True)
def fresh_registry():
    """Inject a clean registry into all relevant modules before each test."""
    fresh = InMemoryJobRegistry()
    import app.state as state_mod
    import app.api.routes.files as routes_mod
    import app.services.compression as comp_mod

    state_mod.job_registry = fresh
    routes_mod.job_registry = fresh
    comp_mod.job_registry = fresh
    yield fresh


@pytest.fixture()
def client(fresh_registry):
    """
    Return a synchronous TestClient for the FastAPI app.

    The compression queue worker start/stop are mocked so tests do not create
    asyncio tasks that would be bound to the wrong event loop across test runs.
    """
    with (
        patch(
            "app.services.compression_queue.compression_queue_service.start",
            new_callable=AsyncMock,
        ),
        patch(
            "app.services.compression_queue.compression_queue_service.stop",
            new_callable=AsyncMock,
        ),
    ):
        with TestClient(app) as c:
            yield c
