"""
In-memory implementation of the job registry.

Stores jobs in a Python dictionary protected by a ``Lock`` to ensure
thread-safe access from multiple threads (the compression worker and HTTP
handlers can access the registry concurrently).

Note: data is not persisted across application restarts.
"""

from threading import Lock

from app.models.job import CompressionJob, JobStatus
from app.state.registry import JobRegistry


class InMemoryJobRegistry(JobRegistry):
    """
    Dictionary-backed in-memory job registry.

    Thread-safe via a ``threading.Lock`` that guards all reads and writes
    to the internal dictionary.
    """

    def __init__(self) -> None:
        self._jobs: dict[str, CompressionJob] = {}
        self._lock = Lock()

    def add(self, job: CompressionJob) -> None:
        """Insert the job into the internal dictionary."""
        with self._lock:
            self._jobs[job.job_id] = job

    def get(self, job_id: str) -> CompressionJob | None:
        """Return the job, or ``None`` if ``job_id`` does not exist."""
        with self._lock:
            return self._jobs.get(job_id)

    def list_all(self) -> list[CompressionJob]:
        """Return a snapshot of all jobs at the time of the call."""
        with self._lock:
            return list(self._jobs.values())

    def update_status(
        self,
        job_id: str,
        status: JobStatus,
        error_message: str | None = None,
    ) -> CompressionJob:
        """
        Update the job status and call ``touch()`` to refresh ``updated_at``.

        Raises:
            KeyError: If ``job_id`` does not exist in the registry.
        """
        with self._lock:
            job = self._jobs[job_id]
            job.status = status
            job.error_message = error_message
            job.touch()
            return job

    def remove(self, job_id: str) -> CompressionJob | None:
        """Remove and return the job, or ``None`` if it did not exist."""
        with self._lock:
            return self._jobs.pop(job_id, None)
