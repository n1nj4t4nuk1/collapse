"""
Abstract interface for the compression job registry.

Defines the contract that all registry implementations must satisfy,
whether backed by in-memory storage, a database, or another persistence layer.
"""

from abc import ABC, abstractmethod

from app.models.job import CompressionJob, JobStatus


class JobRegistry(ABC):
    """Contract for storing and managing compression jobs."""

    @abstractmethod
    def add(self, job: CompressionJob) -> None:
        """Register a new job in the store."""
        raise NotImplementedError

    @abstractmethod
    def get(self, job_id: str) -> CompressionJob | None:
        """Return the job for ``job_id``, or ``None`` if it does not exist."""
        raise NotImplementedError

    @abstractmethod
    def list_all(self) -> list[CompressionJob]:
        """Return a list of all registered jobs."""
        raise NotImplementedError

    @abstractmethod
    def update_status(
        self,
        job_id: str,
        status: JobStatus,
        error_message: str | None = None,
    ) -> CompressionJob:
        """
        Update the status of an existing job.

        Args:
            job_id: Identifier of the job to update.
            status: New status to set.
            error_message: Optional error message (only relevant for FAILED status).

        Returns:
            The updated job.
        """
        raise NotImplementedError

    @abstractmethod
    def remove(self, job_id: str) -> CompressionJob | None:
        """
        Remove the job from the registry.

        Returns:
            The removed job, or ``None`` if it did not exist.
        """
        raise NotImplementedError
