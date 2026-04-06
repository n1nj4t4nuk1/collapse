"""Port for the compression job registry."""

from abc import abstractmethod

from app.domain.interface import Interface
from app.domain.models.job import CompressionJob, JobStatus


class JobRegistryPort(Interface):
    """Contract for storing and managing compression jobs."""

    @abstractmethod
    def add(self, job: CompressionJob) -> None:
        """Register a new job in the store."""

    @abstractmethod
    def get(self, job_id: str) -> CompressionJob | None:
        """Return the job for ``job_id``, or ``None`` if it does not exist."""

    @abstractmethod
    def list_all(self) -> list[CompressionJob]:
        """Return a list of all registered jobs."""

    @abstractmethod
    def update_status(
        self,
        job_id: str,
        status: JobStatus,
        error_message: str | None = None,
    ) -> CompressionJob:
        """Update the status of an existing job.

        Args:
            job_id: Identifier of the job to update.
            status: New status to set.
            error_message: Optional error message (only relevant for FAILED status).

        Returns:
            The updated job.
        """

    @abstractmethod
    def remove(self, job_id: str) -> CompressionJob | None:
        """Remove the job from the registry.

        Returns:
            The removed job, or ``None`` if it did not exist.
        """
