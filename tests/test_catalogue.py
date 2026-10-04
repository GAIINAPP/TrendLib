"""The catalogue is generated from the oracle, so it must not be edited by hand.

A wrong row here becomes a wrong `spec.yaml`, which becomes a function that is
documented as one thing and computes another. The generator is cheap to re-run;
this makes sure nobody has to remember to.
"""

import re
import subprocess
import sys

import pytest

talib = pytest.importorskip("talib", reason="the TA-Lib oracle is not installed here")

ROW = re.compile(r"^\| `([a-z0-9_]+)`")


@pytest.fixture(scope="module")
def catalogue(repo_root):
    return (repo_root / "docs" / "INDICATORS_TALIB.md").read_text()


@pytest.fixture(scope="module")
def catalogue_names(catalogue):
    return [m.group(1) for line in catalogue.splitlines() if (m := ROW.match(line))]


def test_the_catalogue_is_up_to_date(repo_root):
    result = subprocess.run(
        [sys.executable, "scripts/catalogue/talib_catalogue.py", "--check"],
        cwd=repo_root,
        capture_output=True,
        text=True,
    )
    assert result.returncode == 0, result.stderr or result.stdout


def test_it_covers_every_function_the_oracle_exposes(catalogue_names):
    assert len(catalogue_names) == len(talib.get_functions())
    assert len(set(catalogue_names)) == len(catalogue_names), "a function is listed twice"


def test_it_carries_the_generated_banner(catalogue):
    assert catalogue.startswith("<!-- @generated")


def test_the_release_set_is_inside_the_approved_set(repo_root, catalogue_names):
    """Every name in INDICATORS.md section 2.1 is approved somewhere."""
    indicators = (repo_root / "docs" / "INDICATORS.md").read_text()
    release = indicators.split("### 2.1 The 0.1 release set")[1].split("## 3.")[0]
    names = set(re.findall(r"`([a-z0-9_]+)`", release))
    # The four TrendLib defines itself are governed by section 3, not the catalogue.
    own = {"cpr", "pivots_traditional", "pivots_camarilla"}
    assert names, "the release set is empty"
    unapproved = names - set(catalogue_names) - own - {"spec.yaml", "ma_type"}
    assert not unapproved, f"not in the catalogue and not in section 3: {sorted(unapproved)}"


def test_every_shipped_indicator_matches_its_catalogue_row(repo_root, catalogue):
    """What is implemented must say what the catalogue says it says."""
    yaml = pytest.importorskip("yaml")
    folder = repo_root / "crates" / "trendlib" / "src" / "indicators"
    for spec_path in sorted(folder.glob("*/spec.yaml")):
        spec = yaml.safe_load(spec_path.read_text())
        alias = spec.get("talib")
        if alias is None:
            continue
        row = next(
            (line for line in catalogue.splitlines() if line.startswith(f"| `{spec['name']}`")),
            None,
        )
        assert row is not None, f"{spec['name']} has a talib: block but no catalogue row"
        if "&dagger;" in row:
            # The catalogue marks the functions INDICATORS.md section 3 governs
            # rather than the oracle: their inputs and parameters are ours, not
            # TA-Lib's, and the row records what TA-Lib has.
            continue
        assert f"`{alias['name']}`" in row, f"{spec['name']}: alias disagrees with the catalogue"
        for param in spec.get("params") or []:
            if param["type"].startswith("enum:"):
                # An enum has names rather than bounds, and the catalogue
                # prints which enum it is.
                enum_name = param["type"].split(":", 1)[1]
                expected = f'`{param["name"]}` "{param["default"]}" ({enum_name})'
            elif "min" in param or "max" in param:
                # A side the oracle does not check is left out of the spec and
                # the catalogue prints "any" in its place.
                low = param.get("min", "any")
                high = param.get("max", "any")
                expected = f"`{param['name']}` {param['default']} [{low}, {high}]"
            else:
                expected = f"`{param['name']}` {param['default']} any"
            assert expected in row, f"{spec['name']}: {param['name']} disagrees with the catalogue"
        for output in spec["outputs"]:
            assert f"`{output['name']}`" in row, f"{spec['name']}: {output['name']} not in the row"


def test_the_progress_list_is_up_to_date(repo_root):
    """The marks are read off the repository, so staleness is the only failure."""
    result = subprocess.run(
        [sys.executable, "scripts/catalogue/progress.py", "--check"],
        cwd=repo_root,
        capture_output=True,
        text=True,
    )
    assert result.returncode == 0, result.stderr or result.stdout


def test_the_progress_list_covers_every_approved_indicator(repo_root, catalogue_names):
    progress = (repo_root / "docs" / "PROGRESS.md").read_text()
    listed = set(re.findall(r"^- \[[x~ ]\] `([a-z0-9_]+)`", progress, re.M))
    missing = set(catalogue_names) - listed
    assert not missing, f"not listed in PROGRESS.md: {sorted(missing)[:10]}"
    # The three TrendLib defines itself are approved too (INDICATORS.md § 3).
    assert {"cpr", "pivots_traditional", "pivots_camarilla"} <= listed
    assert len(listed) == len(catalogue_names) + 3


def test_nothing_is_ticked_that_is_not_actually_there(repo_root):
    """A tick has to mean four files on disk and a Python entry point."""
    progress = (repo_root / "docs" / "PROGRESS.md").read_text()
    folder = repo_root / "crates" / "trendlib" / "src" / "indicators"
    functions = (repo_root / "python" / "trendlib" / "_functions.py").read_text()
    for name in re.findall(r"^- \[x\] `([a-z0-9_]+)`", progress, re.M):
        for required in ("spec.yaml", "mod.rs", "doc.md"):
            assert (folder / name / required).is_file(), f"{name} is ticked but has no {required}"
        assert list((folder / name / "golden").glob("*.csv")), f"{name} is ticked with no golden"
        assert f"def {name}(" in functions, f"{name} is ticked but is not callable from Python"
