"""Tests for StorageService."""

import pytest
from pathlib import Path
from unittest.mock import AsyncMock, MagicMock

from app.models.job import CompressionAlgorithm
from app.services.storage import StorageService


@pytest.fixture()
def svc(tmp_path, monkeypatch) -> StorageService:
    """Return a StorageService pointing at tmp_path for storage."""
    import app.core.config as cfg
    monkeypatch.setattr(cfg, "INPUT_DIR", tmp_path / "input")
    monkeypatch.setattr(cfg, "OUTPUT_DIR", tmp_path / "output")
    # Re-import so the service picks up patched constants
    import importlib
    import app.services.storage as storage_mod
    importlib.reload(storage_mod)
    return storage_mod.StorageService()


class TestEnsureDirectories:
    def test_creates_directories(self, svc, tmp_path):
        import app.core.config as cfg
        svc.ensure_directories()
        assert cfg.INPUT_DIR.exists()
        assert cfg.OUTPUT_DIR.exists()

    def test_idempotent(self, svc):
        svc.ensure_directories()
        svc.ensure_directories()  # should not raise


class TestBuildInputPath:
    def test_returns_path_in_input_dir(self, svc, tmp_path):
        import app.core.config as cfg
        path = svc.build_input_path("document.pdf")
        assert path.parent == cfg.INPUT_DIR

    def test_preserves_extension(self, svc):
        path = svc.build_input_path("archive.tar.gz")
        assert path.suffix == ".gz"

    def test_unique_per_call(self, svc):
        p1 = svc.build_input_path("file.txt")
        p2 = svc.build_input_path("file.txt")
        assert p1 != p2


class TestBuildOutputPath:
    def test_zip_extension(self, svc, tmp_path):
        import app.core.config as cfg
        path = svc.build_output_path("job1", CompressionAlgorithm.ZIP)
        assert path.suffix == ".zip"
        assert path.parent == cfg.OUTPUT_DIR

    def test_7z_extension(self, svc, tmp_path):
        import app.core.config as cfg
        path = svc.build_output_path("job1", CompressionAlgorithm.SEVENZ)
        assert path.suffix == ".7z"

    def test_filename_is_job_id(self, svc):
        path = svc.build_output_path("myjobid", CompressionAlgorithm.ZIP)
        assert path.stem == "myjobid"


class TestDeleteFile:
    def test_returns_true_when_file_exists(self, svc, tmp_path):
        f = tmp_path / "file.txt"
        f.write_text("data")
        assert svc.delete_file(f) is True
        assert not f.exists()

    def test_returns_false_when_file_missing(self, svc, tmp_path):
        assert svc.delete_file(tmp_path / "ghost.txt") is False
