"""spec.yaml is the source of truth; nothing else may quietly disagree with it.

M1 writes the bindings and wrappers by hand, so a default or a range can exist
in three places at once. `cargo xtask generate` removes that risk in M2; until
then this suite is what stands between a typo and a wrong number shipped under
the right name.
"""

import inspect
import re

import pytest

import trendlib as tl
from trendlib import _core

yaml = pytest.importorskip("yaml")

INDICATORS = ("sma", "ema", "rsi")
GROUPS = ("overlap", "momentum", "volatility", "volume", "levels", "patterns")
PLOT_HINTS = (
    "line",
    "histogram",
    "upper_band",
    "middle_band",
    "lower_band",
    "level",
    "direction",
    "pattern",
)


@pytest.fixture(scope="module")
def indicators_dir(repo_root):
    return repo_root / "crates" / "trendlib" / "src" / "indicators"


@pytest.fixture
def spec(indicators_dir, indicator):
    return yaml.safe_load((indicators_dir / indicator / "spec.yaml").read_text())


def test_every_indicator_folder_has_the_four_required_files(indicators_dir, indicator):
    folder = indicators_dir / indicator
    for name in ("spec.yaml", "mod.rs", "doc.md"):
        assert (folder / name).is_file(), f"{indicator} has no {name}"
    assert (folder / "golden" / "default.csv").is_file()


def test_the_spec_name_equals_the_folder_name(spec, indicator):
    assert spec["name"] == indicator


def test_the_spec_group_is_a_known_group(spec, indicators_dir):
    keys = {g["key"] for g in yaml.safe_load((indicators_dir / "_groups.yaml").read_text())}
    assert keys == set(GROUPS)
    assert spec["group"] in keys


def test_the_python_default_comes_from_the_spec(spec, indicator):
    signature = inspect.signature(getattr(tl, indicator))
    for param in spec["params"]:
        assert signature.parameters[param["name"]].default == param["default"], param["name"]
        assert signature.parameters[param["name"]].kind is inspect.Parameter.KEYWORD_ONLY


def test_the_extension_ranges_come_from_the_spec(spec, indicator):
    exposed = _core.PARAMS[indicator]
    for param in spec["params"]:
        assert exposed[param["name"]] == {
            "default": param["default"],
            "min": param["min"],
            "max": param["max"],
        }, param["name"]


def test_the_documented_range_is_the_enforced_range(spec, indicator):
    function = getattr(tl, indicator)
    for param in spec["params"]:
        name, low, high = param["name"], param["min"], param["max"]
        for bad in (low - 1, high + 1):
            with pytest.raises(tl.InvalidInput) as caught:
                function([1.0, 2.0, 3.0], **{name: bad})
            assert str(caught.value) == f"{indicator}: {name}={bad} is out of range [{low}, {high}]"
        function([1.0, 2.0, 3.0], **{name: low})
        function([1.0, 2.0, 3.0], **{name: high})


def test_outputs_are_named_and_plotted_as_the_spec_says(spec, indicator, closes):
    pd = pytest.importorskip("pandas")
    assert len(spec["outputs"]) == 1, "M1 ships single-output indicators only"
    output = spec["outputs"][0]
    assert output["dtype"] == "float64"
    assert output["plot"] in PLOT_HINTS
    series = getattr(tl, indicator)(pd.Series(closes))
    assert series.name == output["name"]


def test_output_names_are_unique_across_the_catalogue(indicators_dir):
    seen = {}
    for folder in sorted(indicators_dir.iterdir()):
        if not (folder / "spec.yaml").exists():
            continue
        spec = yaml.safe_load((folder / "spec.yaml").read_text())
        inputs = {i["name"] for i in spec["inputs"]}
        for output in spec["outputs"]:
            assert output["name"] not in seen, f"{output['name']} is used twice"
            assert output["name"] not in inputs, f"{output['name']} clashes with an input"
            seen[output["name"]] = spec["name"]


def test_the_talib_alias_matches_the_spec(spec, indicator):
    alias = spec["talib"]
    assert hasattr(tl, alias["name"])
    renamed = inspect.signature(getattr(tl, alias["name"])).parameters
    for ours, theirs in alias["params"].items():
        assert theirs in renamed, f"{alias['name']} has no parameter {theirs}"
        spec_default = next(p["default"] for p in spec["params"] if p["name"] == ours)
        assert renamed[theirs].default == spec_default


def test_the_doc_has_its_sections_in_order(indicators_dir, indicator):
    text = (indicators_dir / indicator / "doc.md").read_text()
    headings = re.findall(r"^## (.+)$", text, flags=re.MULTILINE)
    assert headings == ["Formula", "Conventions", "Example", "References"]
    assert text.startswith("# ")
    summary = text.split("\n\n")[1]
    assert len(summary.split()) < 60, "the summary paragraph becomes a docstring; keep it short"


def test_the_doc_summary_is_descriptive_not_advisory(indicators_dir, indicator):
    text = (indicators_dir / indicator / "doc.md").read_text().lower()
    for word in (" buy ", " sell ", "entry point", "exit point", "recommendation"):
        assert word not in text, f"{indicator}/doc.md reads as advice: {word!r}"


def test_the_golden_header_params_match_the_case(indicators_dir, indicator):
    folder = indicators_dir / indicator / "golden"
    spec = yaml.safe_load((indicators_dir / indicator / "spec.yaml").read_text())
    defaults = {p["name"]: p["default"] for p in spec["params"]}
    minimums = {p["name"]: p["min"] for p in spec["params"]}
    for path in sorted(folder.glob("*.csv")):
        header = dict(
            line[2:].split(": ", 1)
            for line in path.read_text().splitlines()
            if line.startswith("# ")
        )
        assert header["indicator"] == indicator
        assert header["case"] == path.stem
        recorded = dict(field.split("=") for field in header["params"].split(", "))
        wanted = defaults if path.stem == "default" else minimums
        assert {k: int(v) for k, v in recorded.items()} == wanted
        assert "ta-lib-python" in header["oracle"] or header["produced_by"] == "manual"
