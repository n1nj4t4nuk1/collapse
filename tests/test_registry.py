"""Tests for InMemoryJobRegistry."""

from pathlib import Path

import pytest

from app.domain.models.job import CompressionAlgorithm, CompressionJob, JobStatus
from app.infrastructure.persistence.in_memory_job_registry import InMemoryJobRegistry


def _make_job(job_id: str = "job1") -> CompressionJob:
    return CompressionJob(
        job_id=job_id,
        original_filename="file.txt",
        archive_filename="file.txt.7z",
        original_path=Path("/tmp/orig.txt"),
        compressed_path=Path("/tmp/out.7z"),
        algorithm=CompressionAlgorithm.SEVENZ,
        level=3,
    )


@pytest.fixture()
def registry() -> InMemoryJobRegistry:
    return InMemoryJobRegistry()


class TestAdd:
    def test_add_and_get(self, registry):
        job = _make_job("j1")
        registry.add(job)
        assert registry.get("j1") is job

    def test_get_missing_returns_none(self, registry):
        assert registry.get("nonexistent") is None


class TestListAll:
    def test_empty(self, registry):
        assert registry.list_all() == []

    def test_returns_all_jobs(self, registry):
        j1 = _make_job("j1")
        j2 = _make_job("j2")
        registry.add(j1)
        registry.add(j2)
        ids = {j.job_id for j in registry.list_all()}
        assert ids == {"j1", "j2"}

    def test_returns_copy(self, registry):
        job = _make_job()
        registry.add(job)
        lst = registry.list_all()
        lst.clear()
        assert len(registry.list_all()) == 1


class TestUpdateStatus:
    def test_updates_status(self, registry):
        job = _make_job()
        registry.add(job)
        updated = registry.update_status(job.job_id, JobStatus.COMPRESSING)
        assert updated.status is JobStatus.COMPRESSING

    def test_sets_error_message_on_failed(self, registry):
        job = _make_job()
        registry.add(job)
        registry.update_status(job.job_id, JobStatus.FAILED, "boom")
        assert registry.get(job.job_id).error_message == "boom"

    def test_clears_error_message_when_none(self, registry):
        job = _make_job()
        registry.add(job)
        registry.update_status(job.job_id, JobStatus.FAILED, "err")
        registry.update_status(job.job_id, JobStatus.COMPLETED, None)
        assert registry.get(job.job_id).error_message is None

    def test_raises_on_missing_job(self, registry):
        with pytest.raises(KeyError):
            registry.update_status("ghost", JobStatus.COMPLETED)


class TestRemove:
    def test_remove_existing(self, registry):
        job = _make_job()
        registry.add(job)
        removed = registry.remove(job.job_id)
        assert removed is job
        assert registry.get(job.job_id) is None

    def test_remove_missing_returns_none(self, registry):
        assert registry.remove("ghost") is None
