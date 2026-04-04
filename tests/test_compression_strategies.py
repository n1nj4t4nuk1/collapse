"""Tests for compression strategies (ZIP and 7z)."""

import zipfile
import tempfile
from pathlib import Path

import py7zr
import pytest

from app.models.job import CompressionAlgorithm
from app.services.compression_strategies.registry import build_strategy_registry
from app.services.compression_strategies.sevenz import SevenZipCompressionStrategy
from app.services.compression_strategies.zip import ZipCompressionStrategy


SAMPLE_CONTENT = b"Hello, Collapse! " * 100


@pytest.fixture()
def source_file(tmp_path: Path) -> Path:
    """Create a temporary source file with some content."""
    f = tmp_path / "sample.txt"
    f.write_bytes(SAMPLE_CONTENT)
    return f


class TestZipStrategy:
    def test_algorithm_property(self):
        assert ZipCompressionStrategy().algorithm is CompressionAlgorithm.ZIP

    def test_creates_valid_zip(self, source_file, tmp_path):
        archive = tmp_path / "out.zip"
        ZipCompressionStrategy().compress(source_file, archive, "sample.txt", level=1)
        assert archive.exists()
        assert zipfile.is_zipfile(archive)

    def test_zip_contains_original_filename(self, source_file, tmp_path):
        archive = tmp_path / "out.zip"
        ZipCompressionStrategy().compress(source_file, archive, "sample.txt", level=1)
        with zipfile.ZipFile(archive) as zf:
            assert "sample.txt" in zf.namelist()

    def test_zip_content_is_preserved(self, source_file, tmp_path):
        archive = tmp_path / "out.zip"
        ZipCompressionStrategy().compress(source_file, archive, "sample.txt", level=3)
        with zipfile.ZipFile(archive) as zf:
            assert zf.read("sample.txt") == SAMPLE_CONTENT

    @pytest.mark.parametrize("level", [1, 2, 3, 4, 5])
    def test_all_levels_produce_valid_zip(self, source_file, tmp_path, level):
        archive = tmp_path / f"out_l{level}.zip"
        ZipCompressionStrategy().compress(source_file, archive, "sample.txt", level=level)
        assert zipfile.is_zipfile(archive)


class TestSevenZipStrategy:
    def test_algorithm_property(self):
        assert SevenZipCompressionStrategy().algorithm is CompressionAlgorithm.SEVENZ

    def test_creates_valid_7z(self, source_file, tmp_path):
        archive = tmp_path / "out.7z"
        SevenZipCompressionStrategy().compress(source_file, archive, "sample.txt", level=1)
        assert archive.exists()
        assert py7zr.is_7zfile(archive)

    def test_7z_contains_original_filename(self, source_file, tmp_path):
        archive = tmp_path / "out.7z"
        SevenZipCompressionStrategy().compress(source_file, archive, "sample.txt", level=1)
        with py7zr.SevenZipFile(archive, "r") as zf:
            assert "sample.txt" in zf.getnames()

    def test_7z_content_is_preserved(self, source_file, tmp_path):
        archive = tmp_path / "out.7z"
        SevenZipCompressionStrategy().compress(source_file, archive, "sample.txt", level=1)
        with py7zr.SevenZipFile(archive, "r") as zf:
            data = zf.read(["sample.txt"])
            assert data["sample.txt"].read() == SAMPLE_CONTENT

    @pytest.mark.parametrize("level", [1, 2, 3, 4, 5])
    def test_all_levels_produce_valid_7z(self, source_file, tmp_path, level):
        archive = tmp_path / f"out_l{level}.7z"
        SevenZipCompressionStrategy().compress(source_file, archive, "sample.txt", level=level)
        assert py7zr.is_7zfile(archive)


class TestStrategyRegistry:
    def test_contains_all_algorithms(self):
        registry = build_strategy_registry()
        assert CompressionAlgorithm.ZIP in registry
        assert CompressionAlgorithm.SEVENZ in registry

    def test_registry_size(self):
        registry = build_strategy_registry()
        assert len(registry) == len(CompressionAlgorithm)
