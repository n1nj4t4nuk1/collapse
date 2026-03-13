from app.state.in_memory_registry import InMemoryJobRegistry
from app.state.registry import JobRegistry

job_registry: JobRegistry = InMemoryJobRegistry()

__all__ = ["JobRegistry", "job_registry"]
