"""Tests for host/port resolution in app.main._parse_args and run()."""

import sys
from unittest.mock import patch

import pytest

import app.main as main_mod


class TestParseArgs:
    def test_defaults_are_none(self):
        with patch.object(sys, "argv", ["prog"]):
            args = main_mod._parse_args()
        assert args.host is None
        assert args.port is None

    def test_cli_host(self):
        with patch.object(sys, "argv", ["prog", "--host", "127.0.0.1"]):
            args = main_mod._parse_args()
        assert args.host == "127.0.0.1"

    def test_cli_port(self):
        with patch.object(sys, "argv", ["prog", "--port", "9000"]):
            args = main_mod._parse_args()
        assert args.port == 9000


class TestRunResolution:
    def _run_with(self, argv, env):
        captured = {}

        def fake_uvicorn_run(app_str, host, port, reload):
            captured["host"] = host
            captured["port"] = port

        with (
            patch.object(sys, "argv", argv),
            patch.dict("os.environ", env, clear=False),
            patch("app.main.uvicorn.run", side_effect=fake_uvicorn_run),
        ):
            main_mod.run()
        return captured

    def test_defaults(self):
        result = self._run_with(
            ["prog"],
            {"COLLAPSE_HOST": "", "COLLAPSE_PORT": ""},
        )
        assert result["host"] == main_mod.DEFAULT_HOST
        assert result["port"] == main_mod.DEFAULT_PORT

    def test_env_overrides_default(self):
        result = self._run_with(
            ["prog"],
            {"COLLAPSE_HOST": "192.168.1.1", "COLLAPSE_PORT": "9999"},
        )
        assert result["host"] == "192.168.1.1"
        assert result["port"] == 9999

    def test_cli_overrides_env(self):
        result = self._run_with(
            ["prog", "--host", "10.0.0.1", "--port", "1234"],
            {"COLLAPSE_HOST": "192.168.1.1", "COLLAPSE_PORT": "9999"},
        )
        assert result["host"] == "10.0.0.1"
        assert result["port"] == 1234
