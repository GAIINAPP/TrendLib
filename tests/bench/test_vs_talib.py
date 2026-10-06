"""`tl.<name>` against `talib.<NAME>` on the same arrays, through pytest-benchmark.

`docs/TESTING.md` section 8. These are marked `bench` and deselected by the
default options, because a million bars through four hundred functions is not
something to run on every edit:

    pytest tests/bench -m bench                  # the default million bars
    pytest tests/bench -m bench --bench-bars 50000

The table the release checklist reads comes from `scripts/bench/compare.py`,
which times the same calls without pytest's per-test overhead. This file exists
so a single indicator can be profiled in isolation while it is being worked on.
"""

import sys
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "scripts" / "bench"))

talib = pytest.importorskip("talib", reason="the TA-Lib oracle is not installed here")
yaml = pytest.importorskip("yaml")

from compare import MATCHES_ORACLE_WITH, build_columns, ma_type_ints, shared  # noqa: E402

import trendlib as tl  # noqa: E402
from trendlib import _core  # noqa: E402

pytestmark = pytest.mark.bench

SHARED = shared(yaml)


@pytest.fixture(scope="module")
def built(request):
    return build_columns(request.config.getoption("--bench-bars"))


@pytest.mark.parametrize("name,alias", SHARED, ids=[name for name, _ in SHARED])
def test_trendlib(benchmark, name, alias, built):
    kinds = list(_core.KINDS[name])
    bars = [built["close" if kind == "series" else kind] for kind in kinds]
    params = {
        param: spec["default"] for param, spec in _core.PARAMS[name].items()
    } | MATCHES_ORACLE_WITH.get(name, {})
    benchmark.group = name
    benchmark(getattr(tl, name), *bars, **params)


@pytest.mark.parametrize("name,alias", SHARED, ids=[name for name, _ in SHARED])
def test_talib(benchmark, name, alias, built):
    kinds = list(_core.KINDS[name])
    bars = [built["close" if kind == "series" else kind] for kind in kinds if kind != "timestamps"]
    renames = alias.get("params") or {}
    numbers = ma_type_ints(yaml)
    params = {
        param: spec["default"] for param, spec in _core.PARAMS[name].items()
    } | MATCHES_ORACLE_WITH.get(name, {})
    oracle_params = {
        renames[key]: numbers[value] if isinstance(value, str) else value
        for key, value in params.items()
        if key in renames
    }
    benchmark.group = name
    benchmark(getattr(talib, alias["name"]), *bars, **oracle_params)
