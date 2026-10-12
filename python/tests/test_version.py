"""The package version comes from the installed distribution metadata."""

import tomllib
from importlib.metadata import version
from pathlib import Path

from packaging.version import Version

import lightcone_sdk


def test_version_matches_installed_metadata_and_pyproject():
    pyproject = tomllib.loads(
        (Path(__file__).resolve().parents[1] / "pyproject.toml").read_text()
    )
    assert lightcone_sdk.__version__ == version("lightcone-sdk")
    assert Version(lightcone_sdk.__version__) == Version(
        pyproject["project"]["version"]
    )
