"""Integration tests for the /files API endpoints."""

import io
import zipfile
from pathlib import Path
from unittest.mock import AsyncMock, patch, MagicMock

import pytest
from fastapi.testclient import TestClient

from app.main import app
from app.models.job import CompressionAlgorithm, CompressionJob, JobStatus
from app.state.in_memory_registry import InMemoryJobRegistry


def _make_job(
    job_id: str = "job1",
    status: JobStatus = JobStatus.COMPLETED,
    algorithm: CompressionAlgorithm = CompressionAlgorithm.ZIP,
    compressed_path: Path | None = None,
    original_path: Path | None = None,
) -> CompressionJob:
    return CompressionJob(
        job_id=job_id,
        original_filename="file.txt",
        archive_filename="file.txt.zip",
        original_path=original_path or Path("/tmp/orig.txt"),
        compressed_path=compressed_path or Path("/tmp/out.zip"),
        algorithm=algorithm,
        level=3,
        status=status,
    )




# ---------------------------------------------------------------------------
# GET /files
# ---------------------------------------------------------------------------

class TestListJobs:
    def test_empty(self, client):
        r = client.get("/files")
        assert r.status_code == 200
        assert r.json() == []

    def test_returns_jobs_ordered_by_created_at(self, client, fresh_registry):
        j1 = _make_job("j1")
        j2 = _make_job("j2")
        fresh_registry.add(j1)
        fresh_registry.add(j2)
        r = client.get("/files")
        assert r.status_code == 200
        ids = [j["job_id"] for j in r.json()]
        assert ids == sorted(ids, key=lambda jid: next(
            job.created_at for job in fresh_registry.list_all() if job.job_id == jid
        ))


# ---------------------------------------------------------------------------
# POST /files
# ---------------------------------------------------------------------------

class TestUploadFile:
    def test_upload_returns_202(self, client):
        with (
            patch("app.api.routes.files.storage_service.save_upload", new_callable=AsyncMock),
            patch("app.api.routes.files.compression_queue_service.enqueue", new_callable=AsyncMock),
        ):
            r = client.post(
                "/files",
                files={"file": ("hello.txt", b"content", "text/plain")},
                data={"algorithm": "zip", "level": "1"},
            )
        assert r.status_code == 202
        body = r.json()
        assert body["filename"] == "hello.txt"
        assert body["algorithm"] == "zip"
        assert body["level"] == 1
        assert body["status"] == "queued"

    def test_upload_empty_filename_returns_error(self, client):
        # An empty filename causes FastAPI multipart validation to fail with 422
        # before the route handler runs.
        r = client.post(
            "/files",
            files={"file": ("", b"content", "text/plain")},
        )
        assert r.status_code in (400, 422)

    def test_upload_level_out_of_range_returns_422(self, client):
        with (
            patch("app.api.routes.files.storage_service.save_upload", new_callable=AsyncMock),
            patch("app.api.routes.files.compression_queue_service.enqueue", new_callable=AsyncMock),
        ):
            r = client.post(
                "/files",
                files={"file": ("f.txt", b"x", "text/plain")},
                data={"level": "6"},
            )
        assert r.status_code == 422

    def test_upload_default_algorithm_is_7z(self, client):
        with (
            patch("app.api.routes.files.storage_service.save_upload", new_callable=AsyncMock),
            patch("app.api.routes.files.compression_queue_service.enqueue", new_callable=AsyncMock),
        ):
            r = client.post(
                "/files",
                files={"file": ("f.txt", b"x", "text/plain")},
            )
        assert r.status_code == 202
        assert r.json()["algorithm"] == "7z"


# ---------------------------------------------------------------------------
# GET /files/{job_id}/status
# ---------------------------------------------------------------------------

class TestGetJobStatus:
    def test_returns_status(self, client, fresh_registry):
        fresh_registry.add(_make_job("j1", status=JobStatus.QUEUED))
        r = client.get("/files/j1/status")
        assert r.status_code == 200
        assert r.json()["status"] == "queued"

    def test_missing_job_returns_404(self, client):
        r = client.get("/files/ghost/status")
        assert r.status_code == 404


# ---------------------------------------------------------------------------
# GET /files/{job_id}/download
# ---------------------------------------------------------------------------

class TestDownloadArchive:
    def test_download_completed_job(self, client, fresh_registry, tmp_path):
        archive = tmp_path / "out.zip"
        archive.write_bytes(b"PK\x03\x04")  # minimal zip magic
        job = _make_job("j1", status=JobStatus.COMPLETED, compressed_path=archive)
        fresh_registry.add(job)
        r = client.get("/files/j1/download")
        assert r.status_code == 200

    def test_download_queued_returns_409(self, client, fresh_registry):
        fresh_registry.add(_make_job("j1", status=JobStatus.QUEUED))
        r = client.get("/files/j1/download")
        assert r.status_code == 409

    def test_download_compressing_returns_409(self, client, fresh_registry):
        fresh_registry.add(_make_job("j1", status=JobStatus.COMPRESSING))
        r = client.get("/files/j1/download")
        assert r.status_code == 409

    def test_download_failed_returns_409(self, client, fresh_registry):
        job = _make_job("j1", status=JobStatus.FAILED)
        job.error_message = "disk full"
        fresh_registry.add(job)
        r = client.get("/files/j1/download")
        assert r.status_code == 409

    def test_download_missing_archive_returns_404(self, client, fresh_registry):
        job = _make_job("j1", status=JobStatus.COMPLETED, compressed_path=Path("/nonexistent/out.zip"))
        fresh_registry.add(job)
        r = client.get("/files/j1/download")
        assert r.status_code == 404

    def test_missing_job_returns_404(self, client):
        r = client.get("/files/ghost/download")
        assert r.status_code == 404


# ---------------------------------------------------------------------------
# DELETE /files/{job_id}
# ---------------------------------------------------------------------------

class TestDeleteJob:
    def test_delete_completed_job(self, client, fresh_registry, tmp_path):
        orig = tmp_path / "orig.txt"
        arc = tmp_path / "out.zip"
        orig.write_text("x")
        arc.write_bytes(b"PK")
        job = _make_job("j1", status=JobStatus.COMPLETED, original_path=orig, compressed_path=arc)
        fresh_registry.add(job)
        r = client.delete("/files/j1")
        assert r.status_code == 200
        assert r.json()["deleted"] is True
        assert not orig.exists()
        assert not arc.exists()

    def test_delete_queued_returns_409(self, client, fresh_registry):
        fresh_registry.add(_make_job("j1", status=JobStatus.QUEUED))
        r = client.delete("/files/j1")
        assert r.status_code == 409

    def test_delete_missing_job_returns_404(self, client):
        r = client.delete("/files/ghost")
        assert r.status_code == 404

    def test_delete_removes_job_from_registry(self, client, fresh_registry, tmp_path):
        job = _make_job("j1", status=JobStatus.COMPLETED)
        fresh_registry.add(job)
        client.delete("/files/j1")
        assert fresh_registry.get("j1") is None


# ---------------------------------------------------------------------------
# DELETE /files/completed
# ---------------------------------------------------------------------------

class TestDeleteCompleted:
    def test_deletes_only_completed_jobs(self, client, fresh_registry, tmp_path):
        orig = tmp_path / "orig.txt"
        arc = tmp_path / "out.zip"
        orig.write_text("x")
        arc.write_bytes(b"PK")

        completed = _make_job("c1", status=JobStatus.COMPLETED, original_path=orig, compressed_path=arc)
        queued = _make_job("q1", status=JobStatus.QUEUED)
        fresh_registry.add(completed)
        fresh_registry.add(queued)

        r = client.delete("/files/completed")
        assert r.status_code == 200
        body = r.json()
        assert body["deleted_jobs"] == 1
        assert fresh_registry.get("c1") is None
        assert fresh_registry.get("q1") is not None

    def test_empty_when_no_completed(self, client, fresh_registry):
        fresh_registry.add(_make_job("q1", status=JobStatus.QUEUED))
        r = client.delete("/files/completed")
        assert r.json()["deleted_jobs"] == 0

    def test_counts_deleted_files(self, client, fresh_registry, tmp_path):
        orig = tmp_path / "orig.txt"
        arc = tmp_path / "out.zip"
        orig.write_text("x")
        arc.write_bytes(b"PK")
        job = _make_job("c1", status=JobStatus.COMPLETED, original_path=orig, compressed_path=arc)
        fresh_registry.add(job)

        r = client.delete("/files/completed")
        assert r.json()["deleted_files"] == 2
