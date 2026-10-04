# Release and packaging

Goal: `pip install trendlib` works on every supported platform with no compiler,
and every release is built and published by CI with no long-lived secrets.

## 1. Versioning

- SemVer. One version for the whole workspace: `[workspace.package] version` in the
  root `Cargo.toml`, inherited by every crate. The Python package reads it via
  maturin (`dynamic = ["version"]`).
- Before 1.0: minor versions may break the API; every break is listed in
  `CHANGELOG.md` under "Changed (breaking)".
- Pre-releases use PEP 440 forms that maturin maps from Cargo: Cargo `0.1.0-alpha.1`
  → wheel `0.1.0a1`.
- `CHANGELOG.md` follows Keep a Changelog. Every user-visible PR adds a line under
  "Unreleased".

## 2. `pyproject.toml`

```toml
[build-system]
requires = ["maturin>=1.7,<2"]
build-backend = "maturin"

[project]
name = "trendlib"
description = "Fast, tested technical-analysis indicators with a Rust core"
readme = "README.md"
requires-python = ">=3.11"
license = "MIT OR Apache-2.0"
authors = [{ name = "GAIIN Technologies" }]
keywords = ["technical-analysis", "indicators", "trading", "ta-lib", "finance", "nse"]
classifiers = [
  "Programming Language :: Python :: 3",
  "Programming Language :: Rust",
  "Topic :: Office/Business :: Financial :: Investment",
  "Intended Audience :: Developers",
  "Intended Audience :: Financial and Insurance Industry",
]
dependencies = ["numpy>=1.26"]
dynamic = ["version"]

[project.optional-dependencies]
pandas = ["pandas>=2.0"]
polars = ["polars>=1.0"]
dev = ["pytest", "hypothesis", "pytest-benchmark", "ruff", "pandas>=2.0", "TA-Lib==0.8.1"]
docs = ["mkdocs-material"]

[project.urls]
Homepage = "https://github.com/GAIINAPP/TrendLib"
Documentation = "https://gaiinapp.github.io/TrendLib/"
Changelog = "https://github.com/GAIINAPP/TrendLib/blob/main/CHANGELOG.md"

[tool.maturin]
manifest-path = "crates/trendlib-py/Cargo.toml"
python-source = "python"
module-name = "trendlib._core"
features = ["pyo3/extension-module", "pyo3/abi3-py311"]
```

Notes for implementation:

- If the installed maturin rejects the SPDX `license` string, use the form it
  accepts and record the choice in `DECISIONS.md`.
- Pin `pyo3` to the newest version that the `numpy` crate supports; they must match.
- The sdist must build with only Rust and Python present. Test it in CI by building
  the sdist, then `pip install dist/*.tar.gz` in a clean job.

## 3. Wheels

`release.yml` builds with `PyO3/maturin-action` (pin every action to its current
major version):

| Platform | Runner | Target | Notes |
| --- | --- | --- | --- |
| Linux glibc | ubuntu | x86_64, aarch64 | `manylinux: auto` |
| Linux musl | ubuntu | x86_64, aarch64 | `manylinux: musllinux_1_2` |
| macOS | macOS arm64 | aarch64, x86_64 (cross) | |
| Windows | windows | x64 | |
| sdist | ubuntu | — | `maturin sdist` |

Because of `abi3-py311`, each platform produces one wheel for every Python ≥ 3.11.

## 4. Publishing (PyPI Trusted Publishing)

No API tokens. GitHub Actions proves its identity to PyPI with OIDC.

**One-time setup (human):**

1. Accounts on pypi.org and test.pypi.org, 2FA on.
2. On each, add a *pending trusted publisher* for project `trendlib`: GitHub
   owner = the org from Q1 (`GAIINAPP` unless it moves), repository `TrendLib`,
   workflow `release.yml`,
   environment `pypi` (on TestPyPI: `testpypi`).
3. In the GitHub repo, create environments `pypi` and `testpypi`. Require a reviewer
   (the maintainer) on `pypi`.

**Workflow shape (`release.yml`):**

- Triggers: push of a tag `v*` → build, publish to TestPyPI, smoke-test, then wait
  for approval on the `pypi` environment and publish to PyPI.
  `workflow_dispatch` with input `target=testpypi` → build + TestPyPI only.
- Jobs: `wheels` (matrix § 3) and `sdist` upload artifacts → `publish-testpypi`
  (environment `testpypi`, `permissions: id-token: write`, `pypa/gh-action-pypi-publish`
  with `repository-url: https://test.pypi.org/legacy/`) → `smoke-testpypi` (matrix
  ubuntu/macOS/windows: `pip install --index-url https://test.pypi.org/simple/
  --extra-index-url https://pypi.org/simple/ trendlib==<version>`, retry for up to 5
  minutes while the index catches up, then run a script that imports `trendlib`,
  computes `tl.rsi` on a fixed array and compares to a hard-coded value) →
  `publish-pypi` (environment `pypi`, same action, default repository) →
  `smoke-pypi` (same smoke test against PyPI).
- The tag must equal `v` + the workspace version; the first job fails otherwise.

## 5. Release checklist

1. `CHANGELOG.md`: move "Unreleased" to the new version with today's date.
2. Bump `[workspace.package] version`.
3. `cargo xtask regen-check` clean; `api/public_api.txt` changes reviewed.
4. Nightly benchmark table: no shared indicator slower than 1.5× TA-Lib.
5. All golden tests active, or each `#[ignore]` listed in the release notes.
6. Merge to `main`, tag `vX.Y.Z`, push the tag.
7. Approve the `pypi` deployment after the TestPyPI smoke test passes.
8. Create the GitHub Release from the CHANGELOG section.

Never re-upload a version: PyPI forbids it. A bad release is yanked on PyPI and
fixed with a new patch version.

## 6. Name reservation

The `trendlib` name on PyPI is only claimed by the first upload. Right after M3
passes, publish `0.1.0a1` (real, working code, not a placeholder) to PyPI.

## 7. crates.io (M6)

Publish the `trendlib` crate only (not `trendlib-py` or `xtask`, which carry
`publish = false`). Prefer crates.io Trusted Publishing (OIDC from GitHub
Actions, like PyPI) if it is available for the account; otherwise use an API token
scoped to `publish-update` for `trendlib`, stored as `CARGO_REGISTRY_TOKEN` in a
protected `crates-io` environment. `cargo publish --dry-run -p trendlib` runs in CI
from M3 on to keep the crate packageable.
