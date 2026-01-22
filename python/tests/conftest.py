"""Pytest configuration and shared fixtures."""

import os

import pytest


def pytest_configure(config):
    """Configure pytest markers."""
    config.addinivalue_line("markers", "unit: Unit tests (run offline)")
    config.addinivalue_line("markers", "devnet: Integration tests against devnet")


def pytest_collection_modifyitems(config, items):
    """Skip devnet tests unless explicitly requested."""
    run_devnet = config.getoption("-k", default="") and "devnet" in config.getoption("-k", default="")

    for item in items:
        if "test_devnet" in str(item.fspath):
            if not run_devnet and "DEVNET_TESTS" not in os.environ:
                item.add_marker(pytest.mark.skip(reason="Devnet tests skipped by default. Set DEVNET_TESTS=1 or use -k devnet"))
