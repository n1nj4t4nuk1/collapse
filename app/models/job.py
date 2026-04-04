"""
Domain models for compression jobs.

Defines the status and algorithm enums, and the dataclass that represents
a compression job with all its metadata.
"""

from dataclasses import dataclass, field
from datetime import datetime, timezone
from enum import Enum
from pathlib import Path


class JobStatus(str, Enum):
    """Lifecycle states of a compression job."""

    QUEUED = "queued"
    COMPRESSING = "compressing"
    COMPLETED = "completed"
    FAILED = "failed"


class CompressionAlgorithm(str, Enum):
    """Compression algorithm supported by the service."""

    ZIP = "zip"
    SEVENZ = "7z"


@dataclass(slots=True)
class CompressionJob:
    """
    Represents a single file compression job.

    Attributes:
        job_id: Unique job identifier (UUID hex string).
        original_filename: Original name of the uploaded file.
        archive_filename: Name of the resulting compressed archive.
        original_path: Absolute path to the original file on disk.
        compressed_path: Absolute path to the compressed archive on disk.
        algorithm: Compression algorithm used for this job.
        level: Compression level (1 = fastest, 5 = maximum compression).
        status: Current lifecycle status of the job.
        error_message: Error details if the job failed, None otherwise.
        created_at: UTC timestamp when the job was created.
        updated_at: UTC timestamp of the last status change.
    """

    job_id: str
    original_filename: str
    archive_filename: str
    original_path: Path
    compressed_path: Path
    algorithm: CompressionAlgorithm
    level: int
    status: JobStatus = JobStatus.QUEUED
    error_message: str | None = None
    created_at: datetime = field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = field(default_factory=lambda: datetime.now(timezone.utc))

    def touch(self) -> None:
        """Update ``updated_at`` to the current UTC time."""
        self.updated_at = datetime.now(timezone.utc)
