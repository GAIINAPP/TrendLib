from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
TESTDATA = REPO_ROOT / "testdata"


@pytest.fixture(scope="session")
def repo_root() -> Path:
    return REPO_ROOT


@pytest.fixture(scope="session")
def testdata() -> Path:
    return TESTDATA
