from threading import Lock

from app.models.job import CompressionJob, JobStatus
from app.state.registry import JobRegistry


class InMemoryJobRegistry(JobRegistry):
    def __init__(self) -> None:
        self._jobs: dict[str, CompressionJob] = {}
        self._lock = Lock()

    def add(self, job: CompressionJob) -> None:
        with self._lock:
            self._jobs[job.job_id] = job

    def get(self, job_id: str) -> CompressionJob | None:
        with self._lock:
            return self._jobs.get(job_id)

    def list_all(self) -> list[CompressionJob]:
        with self._lock:
            return list(self._jobs.values())

    def update_status(
        self,
        job_id: str,
        status: JobStatus,
        error_message: str | None = None,
    ) -> CompressionJob:
        with self._lock:
            job = self._jobs[job_id]
            job.status = status
            job.error_message = error_message
            job.touch()
            return job

    def remove(self, job_id: str) -> CompressionJob | None:
        with self._lock:
            return self._jobs.pop(job_id, None)
