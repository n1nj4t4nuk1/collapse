from abc import ABC, abstractmethod

from app.models.job import CompressionJob, JobStatus


class JobRegistry(ABC):
    @abstractmethod
    def add(self, job: CompressionJob) -> None:
        raise NotImplementedError

    @abstractmethod
    def get(self, job_id: str) -> CompressionJob | None:
        raise NotImplementedError

    @abstractmethod
    def list_all(self) -> list[CompressionJob]:
        raise NotImplementedError

    @abstractmethod
    def update_status(
        self,
        job_id: str,
        status: JobStatus,
        error_message: str | None = None,
    ) -> CompressionJob:
        raise NotImplementedError

    @abstractmethod
    def remove(self, job_id: str) -> CompressionJob | None:
        raise NotImplementedError
