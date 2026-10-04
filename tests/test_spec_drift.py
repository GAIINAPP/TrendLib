"""spec.yaml is the source of truth; nothing else may quietly disagree with it.

Checked for every indicator in the tree, not a chosen few, because the one that
drifts will be the one nobody listed.
"""

import inspect
import re

import pytest

import trendlib as tl
from trendlib import _core

yaml = pytest.importorskip("yaml")

PLOT_HINTS = {
    "line",
    "histogram",
    "upper_band",
    "middle_band",
    "lower_band",
    "level",
    "direction",
    "pattern",
}
DTYPES = {"float64", "int32"}


@pytest.fixture(scope="module")
def indicators_dir(repo_root):
    return repo_root / "crates" / "trendlib" / "src" / "indicators"


@pytest.fixture(scope="module")
def groups(indicators_dir):
    return {g["key"] for g in yaml.safe_load((indicators_dir / "_groups.yaml").read_text())}


@pytest.fixture
def spec(indicators_dir, indicator):
    return yaml.safe_load((indicators_dir / indicator.name / "spec.yaml").read_text())


def test_every_indicator_folder_has_the_required_files(indicators_dir, indicator):
    folder = indicators_dir / indicator.name
    for name in ("spec.yaml", "mod.rs", "doc.md"):
        assert (folder / name).is_file(), f"{indicator.name} has no {name}"
    assert (folder / "golden" / "default.csv").is_file()


def test_the_spec_name_equals_the_folder_name(spec, indicator):
    assert spec["name"] == indicator.name


def test_the_spec_group_is_a_known_group(spec, groups, indicator):
    assert spec["group"] in groups
    assert spec["group"] == indicator.group


def test_inputs_match_what_the_extension_reports(spec, indicator):
    assert [i["name"] for i in spec["inputs"]] == indicator.inputs


def test_outputs_match_what_the_extension_reports(spec, indicator):
    assert [o["name"] for o in spec["outputs"]] == indicator.outputs
    for output in spec["outputs"]:
        assert output["dtype"] in DTYPES
        assert output["plot"] in PLOT_HINTS


def test_the_python_default_comes_from_the_spec(spec, indicator):
    signature = inspect.signature(getattr(tl, indicator.name))
    for param in spec.get("params") or []:
        found = signature.parameters[param["name"]]
        assert found.default == param["default"], param["name"]
        assert found.kind is inspect.Parameter.KEYWORD_ONLY


def test_the_extension_ranges_come_from_the_spec(spec, indicator):
    exposed = _core.PARAMS[indicator.name]
    declared = spec.get("params") or []
    assert set(exposed) == {p["name"] for p in declared}
    for param in declared:
        assert exposed[param["name"]]["default"] == param["default"]
        assert exposed[param["name"]]["min"] == param.get("min")
        assert exposed[param["name"]]["max"] == param.get("max")


def test_output_names_are_unique_across_the_catalogue(indicators_dir):
    seen: dict[str, str] = {}
    for path in sorted(indicators_dir.glob("*/spec.yaml")):
        spec = yaml.safe_load(path.read_text())
        inputs = {i["name"] for i in spec["inputs"]}
        for output in spec["outputs"]:
            assert output["name"] not in seen, (
                f"{output['name']} is claimed by both {spec['name']} and {seen[output['name']]}"
            )
            assert output["name"] not in inputs
            seen[output["name"]] = spec["name"]


def test_the_talib_alias_matches_the_spec(spec, indicator):
    alias = spec.get("talib")
    if alias is None:
        pytest.skip("no TA-Lib equivalent")
    assert hasattr(tl, alias["name"])
    renamed = inspect.signature(getattr(tl, alias["name"])).parameters
    for ours, theirs in (alias.get("params") or {}).items():
        assert theirs in renamed, f"{alias['name']} has no parameter {theirs}"
        spec_default = next(p["default"] for p in spec["params"] if p["name"] == ours)
        assert renamed[theirs].default == spec_default


def test_the_doc_has_its_sections_in_order(indicators_dir, indicator):
    text = (indicators_dir / indicator.name / "doc.md").read_text()
    headings = re.findall(r"^## (.+)$", text, flags=re.MULTILINE)
    assert headings == ["Formula", "Conventions", "Example", "References"]
    assert text.startswith("# ")
    summary = text.split("\n\n")[1]
    assert len(summary.split()) < 60, "the summary becomes a docstring; keep it short"


def test_the_doc_is_descriptive_not_advisory(indicators_dir, indicator):
    text = (indicators_dir / indicator.name / "doc.md").read_text().lower()
    for word in (" buy ", " sell ", "entry point", "exit point", "recommendation"):
        assert word not in text, f"{indicator.name}/doc.md reads as advice: {word!r}"


def test_the_golden_header_params_match_the_case(indicators_dir, indicator):
    folder = indicators_dir / indicator.name / "golden"
    spec = yaml.safe_load((indicators_dir / indicator.name / "spec.yaml").read_text())
    declared = spec.get("params") or []
    for path in sorted(folder.glob("*.csv")):
        header = dict(
            line[2:].split(": ", 1)
            for line in path.read_text().splitlines()
            if line.startswith("# ")
        )
        assert header["indicator"] == indicator.name
        assert header["case"] == path.stem
        assert "ta-lib-python" in header["oracle"] or header["produced_by"] == "manual"
        if not declared:
            assert header["params"] == "none"
            continue
        recorded = dict(field.split("=") for field in header["params"].split(", "))
        assert set(recorded) == {p["name"] for p in declared}
