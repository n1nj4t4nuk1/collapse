"""
File storage service.

Handles directory creation, input/output path generation,
streaming uploads to disk, and file deletion.
"""

from pathlib import Path
from uuid import uuid4

from fastapi import UploadFile

from app.core.config import INPUT_DIR, MAX_UPLOAD_CHUNK_SIZE, OUTPUT_DIR
from app.models.job import CompressionAlgorithm


class StorageService:
    """Encapsulates all file I/O operations for the service."""

    def ensure_directories(self) -> None:
        """Create the input and output storage directories if they do not exist."""
        INPUT_DIR.mkdir(parents=True, exist_ok=True)
        OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    def build_input_path(self, filename: str) -> Path:
        """
        Generate a unique path in the input directory for an uploaded file.

        A UUID hex prefix is used to avoid collisions while preserving the original extension.
        """
        suffix = Path(filename).suffix
        return INPUT_DIR / f"{uuid4().hex}{suffix}"

    def build_output_path(self, job_id: str, algorithm: CompressionAlgorithm) -> Path:
        """
        Generate the output path for a job's compressed archive.

        The filename is the ``job_id`` with the algorithm extension (.zip or .7z).
        """
        ext = "zip" if algorithm is CompressionAlgorithm.ZIP else "7z"
        return OUTPUT_DIR / f"{job_id}.{ext}"

    async def save_upload(self, upload: UploadFile, destination: Path) -> None:
        """
        Write an uploaded file to disk by reading it in chunks.

        Reads ``MAX_UPLOAD_CHUNK_SIZE`` bytes at a time to avoid loading the
        entire file into memory at once.
        """
        self.ensure_directories()

        with destination.open("wb") as target:
            while chunk := await upload.read(MAX_UPLOAD_CHUNK_SIZE):
                target.write(chunk)

        await upload.close()

    def delete_file(self, path: Path) -> bool:
        """
        Delete the file at ``path`` if it exists.

        Returns:
            True if the file existed and was deleted, False otherwise.
        """
        if not path.exists():
            return False

        path.unlink()
        return True


storage_service = StorageService()
