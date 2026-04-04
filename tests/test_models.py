"""Tests for domain models (CompressionJob, JobStatus, CompressionAlgorithm)."""

from datetime import datetime, timezone
from pathlib import Path

import pytest

from app.models.job import CompressionAlgorithm, CompressionJob, JobStatus


def _make_job(**overrides) -> CompressionJob:
    defaults = dict(
        job_id="abc123",
        original_filename="file.txt",
        archive_filename="file.txt.7z",
        original_path=Path("/tmp/original.txt"),
        compressed_path=Path("/tmp/archive.7z"),
        algorithm=CompressionAlgorithm.SEVENZ,
        level=3,
    )
    defaults.update(overrides)
    return CompressionJob(**defaults)


class TestJobStatus:
    def test_values(self):
        assert JobStatus.QUEUED == "queued"
        assert JobStatus.COMPRESSING == "compressing"
        assert JobStatus.COMPLETED == "completed"
        assert JobStatus.FAILED == "failed"


class TestCompressionAlgorithm:
    def test_values(self):
        assert CompressionAlgorithm.ZIP == "zip"
        assert CompressionAlgorithm.SEVENZ == "7z"


class TestCompressionJob:
    def test_defaults(self):
        job = _make_job()
        assert job.status is JobStatus.QUEUED
        assert job.error_message is None
        assert isinstance(job.created_at, datetime)
        assert isinstance(job.updated_at, datetime)

    def test_touch_updates_updated_at(self):
        job = _make_job()
        before = job.updated_at
        job.touch()
        assert job.updated_at >= before

    def test_touch_does_not_change_created_at(self):
        job = _make_job()
        original_created = job.created_at
        job.touch()
        assert job.created_at == original_created

    def test_timestamps_are_utc(self):
        job = _make_job()
        assert job.created_at.tzinfo == timezone.utc
        assert job.updated_at.tzinfo == timezone.utc
