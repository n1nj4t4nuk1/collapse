"""Tests for FilesystemStorage."""

import io
from pathlib import Path

import pytest

from app.domain.models.job import CompressionAlgorithm
from app.infrastructure.storage.filesystem_storage import FilesystemStorage


@pytest.fixture()
def svc(tmp_path) -> FilesystemStorage:
    """Return a FilesystemStorage pointing at tmp_path for storage."""
    return FilesystemStorage(
        input_dir=tmp_path / "input",
        output_dir=tmp_path / "output",
    )


class TestEnsureDirectories:
    def test_creates_directories(self, svc, tmp_path):
        svc.ensure_directories()
        assert (tmp_path / "input").exists()
        assert (tmp_path / "output").exists()

    def test_idempotent(self, svc):
        svc.ensure_directories()
        svc.ensure_directories()  # should not raise


class TestBuildInputPath:
    def test_returns_path_in_input_dir(self, svc, tmp_path):
        path = svc.build_input_path("document.pdf")
        assert path.parent == tmp_path / "input"

    def test_preserves_extension(self, svc):
        path = svc.build_input_path("archive.tar.gz")
        assert path.suffix == ".gz"

    def test_unique_per_call(self, svc):
        p1 = svc.build_input_path("file.txt")
        p2 = svc.build_input_path("file.txt")
        assert p1 != p2


class TestBuildOutputPath:
    def test_zip_extension(self, svc, tmp_path):
        path = svc.build_output_path("job1", CompressionAlgorithm.ZIP)
        assert path.suffix == ".zip"
        assert path.parent == tmp_path / "output"

    def test_7z_extension(self, svc, tmp_path):
        path = svc.build_output_path("job1", CompressionAlgorithm.SEVENZ)
        assert path.suffix == ".7z"

    def test_filename_is_job_id(self, svc):
        path = svc.build_output_path("myjobid", CompressionAlgorithm.ZIP)
        assert path.stem == "myjobid"


class TestSaveFile:
    def test_saves_content(self, svc, tmp_path):
        svc.ensure_directories()
        dest = tmp_path / "input" / "test.txt"
        source = io.BytesIO(b"hello world")
        svc.save_file(source, dest)
        assert dest.read_bytes() == b"hello world"

    def test_creates_directories_if_missing(self, svc, tmp_path):
        dest = tmp_path / "input" / "test.txt"
        source = io.BytesIO(b"data")
        svc.save_file(source, dest)
        assert dest.exists()


class TestDeleteFile:
    def test_returns_true_when_file_exists(self, svc, tmp_path):
        f = tmp_path / "file.txt"
        f.write_text("data")
        assert svc.delete_file(f) is True
        assert not f.exists()

    def test_returns_false_when_file_missing(self, svc, tmp_path):
        assert svc.delete_file(tmp_path / "ghost.txt") is False
