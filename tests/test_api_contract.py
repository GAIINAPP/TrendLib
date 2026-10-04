"""Every rule in docs/PYTHON_API.md, checked against every indicator.

The `indicator` fixture enumerates the whole catalogue, so each rule below is
written once and holds for all of them.
"""

import numpy as np
import pytest
from conftest import warm_up

import trendlib as tl

pd = pytest.importorskip("pandas")


def outputs_of(result, indicator):
    """The result as a list of arrays, whatever shape it came back in."""
    if indicator.multi_output:
        assert isinstance(result, tuple)
        assert len(result) == len(indicator.outputs)
        return list(result)
    assert not isinstance(result, tuple)
    return [result]


def test_numpy_in_numpy_out(bars, indicator):
    for column in outputs_of(indicator.call(bars), indicator):
        assert isinstance(column, np.ndarray)
        assert column.dtype in (np.float64, np.int32)
        assert column.shape == bars["close"].shape


def test_list_in_numpy_out(bars, indicator):
    import trendlib as tl

    columns = [list(column[:60]) for column in indicator.columns(bars, 60)]
    result = getattr(tl, indicator.name)(*columns)
    for column in outputs_of(result, indicator):
        assert isinstance(column, np.ndarray)
        assert column.shape == (60,)


def test_pandas_series_keeps_its_index_and_output_names(bars, indicator):
    index = pd.date_range("2024-01-01", periods=len(bars["close"]), freq="D", tz="Asia/Kolkata")
    series = [pd.Series(column, index=index) for column in indicator.columns(bars)]
    result = getattr(tl, indicator.name)(*series)

    if indicator.multi_output:
        assert isinstance(result, pd.DataFrame)
        assert list(result.columns) == indicator.outputs
        assert result.index.equals(index)
    else:
        assert isinstance(result, pd.Series)
        assert result.name == indicator.outputs[0]
        assert result.index.equals(index)


def _interchangeable_inputs(indicator) -> bool:
    """Two or more generic series, which a frame cannot tell apart."""
    return sum(1 for name in indicator.inputs if name.startswith("source")) > 1


def test_a_dataframe_is_refused_when_it_cannot_say_which_column_is_which(bars, indicator):
    if not _interchangeable_inputs(indicator):
        pytest.skip("inputs are named columns, so a frame is unambiguous")
    with pytest.raises(tl.InvalidInput, match="cannot say which column"):
        getattr(tl, indicator.name)(indicator.frame(bars))


def test_a_dataframe_stands_in_for_every_input(bars, indicator):
    if _interchangeable_inputs(indicator):
        pytest.skip("a frame is refused for interchangeable inputs")
    frame = indicator.frame(bars)
    from_frame = getattr(tl, indicator.name)(frame)
    from_columns = getattr(tl, indicator.name)(
        *[pd.Series(column) for column in indicator.columns(bars)]
    )
    if indicator.multi_output:
        pd.testing.assert_frame_equal(from_frame, from_columns)
    else:
        pd.testing.assert_series_equal(from_frame, from_columns)


def test_column_matching_ignores_case(bars, indicator):
    if _interchangeable_inputs(indicator):
        pytest.skip("a frame is refused for interchangeable inputs")
    frame = indicator.frame(bars)
    shouted = frame.rename(columns={name: name.upper() for name in frame.columns})
    result = getattr(tl, indicator.name)(shouted)
    expected = getattr(tl, indicator.name)(frame)
    if indicator.multi_output:
        pd.testing.assert_frame_equal(result, expected)
    else:
        pd.testing.assert_series_equal(result, expected)


def test_a_frame_without_the_columns_names_what_is_missing(bars, indicator):
    if _interchangeable_inputs(indicator):
        pytest.skip("a frame is refused for interchangeable inputs")
    frame = indicator.frame(bars)
    wanted = "close" if indicator.inputs[0].startswith("source") else indicator.inputs[0]
    with pytest.raises(tl.InvalidInput, match=f"no '{wanted}' column"):
        getattr(tl, indicator.name)(frame.drop(columns=[wanted]))


def test_a_missing_input_says_which_one(bars, indicator):
    if len(indicator.inputs) < 2:
        pytest.skip("only one input to leave out")
    columns = indicator.columns(bars)
    with pytest.raises(tl.InvalidInput, match="missing input"):
        getattr(tl, indicator.name)(columns[0])


def test_parameters_are_keyword_only(bars, indicator):
    if not indicator.params:
        pytest.skip("no parameters")
    columns = indicator.columns(bars, 60)
    with pytest.raises(TypeError):
        getattr(tl, indicator.name)(*columns, 10)


def test_an_out_of_range_parameter_names_the_range(bars, indicator):
    columns = indicator.columns(bars, 60)
    for name, spec in indicator.params.items():
        if spec["min"] is None:
            continue
        for bad in (spec["min"] - 1, spec["max"] + 1):
            with pytest.raises(tl.InvalidInput) as caught:
                getattr(tl, indicator.name)(*columns, **{name: bad})
            assert str(caught.value) == (
                f"{indicator.name}: {name}={bad} is out of range [{spec['min']}, {spec['max']}]"
            )


def test_a_parameter_of_the_wrong_type_is_rejected(bars, indicator):
    """A name where a number belongs, or a number where a name belongs."""
    columns = indicator.columns(bars, 60)
    for name, spec in indicator.params.items():
        if spec["choices"]:
            wrong, expected = 3, "must be a string"
        else:
            wrong, expected = "nonsense", r"must be an integer|must be a number"
        with pytest.raises(tl.InvalidInput, match=expected):
            getattr(tl, indicator.name)(*columns, **{name: wrong})


def test_an_unknown_choice_lists_the_ones_that_work(bars, indicator):
    columns = indicator.columns(bars, 60)
    for name, spec in indicator.params.items():
        if not spec["choices"]:
            continue
        with pytest.raises(tl.InvalidInput) as caught:
            getattr(tl, indicator.name)(*columns, **{name: "nonsense"})
        message = str(caught.value)
        assert message.startswith(f"{indicator.name}: {name} must be one of ")
        for choice in spec["choices"]:
            assert choice in message, message


def test_an_average_that_is_not_built_yet_is_refused_by_name(repo_root, bars, indicator):
    """INDICATORS.md section 1: a value the contract has but the code does not
    is an error that says so, never a quiet stand-in for a different average."""
    import yaml

    columns = indicator.columns(bars, 60)
    enums = yaml.safe_load((repo_root / "crates/trendlib/src/indicators/_enums.yaml").read_text())
    for name, spec in indicator.params.items():
        if not spec["choices"]:
            continue
        for value in enums["MaType"]["values"]:
            if value in spec["choices"]:
                continue
            with pytest.raises(tl.InvalidInput, match="not implemented yet"):
                getattr(tl, indicator.name)(*columns, **{name: value})


def test_errors_are_value_errors():
    assert issubclass(tl.InvalidInput, ValueError)
    assert issubclass(tl.InsufficientHistory, ValueError)
    assert not issubclass(tl.TrendLibError, ValueError)


def test_a_nan_after_the_first_valid_bar_names_the_input_and_row(bars, indicator):
    for position, name in enumerate(indicator.inputs):
        columns = [column.copy() for column in indicator.columns(bars, 100)]
        columns[position][42] = np.nan
        with pytest.raises(tl.InvalidInput) as caught:
            getattr(tl, indicator.name)(*columns)
        assert name in str(caught.value)
        assert "row 42" in str(caught.value)


def test_leading_nan_rows_are_skipped(bars, indicator):
    trimmed = outputs_of(indicator.call(bars, 300), indicator)
    padded = getattr(tl, indicator.name)(
        *[np.concatenate([[np.nan] * 4, column]) for column in indicator.columns(bars, 300)]
    )
    for shifted, plain in zip(outputs_of(padded, indicator), trimmed, strict=True):
        assert warm_up(shifted[:4]).all()
        if indicator.absolute_index:
            # A row index points into the caller's own array, so a warm-up
            # prefix moves it by exactly its own length.
            expected = np.where(warm_up(plain), plain, plain + 4)
            assert np.array_equal(shifted[4:], expected, equal_nan=True)
        else:
            assert np.array_equal(shifted[4:], plain, equal_nan=True)


def test_empty_input_returns_empty_output(indicator):
    empty = [np.array([], dtype=np.float64) for _ in indicator.inputs]
    for column in outputs_of(getattr(tl, indicator.name)(*empty), indicator):
        assert column.shape == (0,)


def test_the_input_is_never_modified(bars, indicator):
    columns = indicator.columns(bars)
    originals = [column.copy() for column in columns]
    getattr(tl, indicator.name)(*columns)
    for column, original in zip(columns, originals, strict=True):
        assert np.array_equal(column, original)


def test_the_output_is_a_new_array(bars, indicator):
    first = outputs_of(indicator.call(bars), indicator)
    second = outputs_of(indicator.call(bars), indicator)
    for a, b in zip(first, second, strict=True):
        assert a is not b
        for column in indicator.columns(bars):
            assert not np.shares_memory(a, column)


def test_a_non_contiguous_input_is_handled(bars, indicator):
    strided = [column[::2] for column in indicator.columns(bars)]
    assert not strided[0].flags["C_CONTIGUOUS"]
    for column in outputs_of(getattr(tl, indicator.name)(*strided), indicator):
        assert column.shape == strided[0].shape


def test_an_integer_input_is_accepted(bars, indicator):
    columns = [column[:80].astype(np.int64) for column in indicator.columns(bars, 80)]
    for column in outputs_of(getattr(tl, indicator.name)(*columns), indicator):
        assert column.dtype in (np.float64, np.int32)


def test_a_two_dimensional_input_is_rejected(indicator):
    square = [np.zeros((4, 4)) for _ in indicator.inputs]
    with pytest.raises(tl.InvalidInput, match="one-dimensional"):
        getattr(tl, indicator.name)(*square)


def test_lookback_is_available_for_every_indicator(indicator):
    assert indicator.lookback() >= 0
    assert indicator.lookback(**indicator.defaults()) == indicator.lookback()


def test_lookback_rejects_an_unknown_name():
    with pytest.raises(tl.InvalidInput, match="no indicator named"):
        tl.lookback("nonesuch", period=5)


def test_lookback_rejects_an_unknown_parameter(indicator):
    with pytest.raises(TypeError, match="no parameter"):
        tl.lookback(indicator.name, nonesuch=1)


def test_public_surface_matches_the_contract():
    from trendlib import _functions

    fixed = {
        "TrendLibError",
        "InvalidInput",
        "InsufficientHistory",
        "__version__",
        "__version_info__",
        "stream",
    }
    assert set(tl.__all__) == fixed | set(_functions.__all__)
    for name in tl.__all__:
        assert hasattr(tl, name), name


def test_the_extension_module_is_not_public():
    assert "_core" not in tl.__all__


def test_every_indicator_is_reachable_and_documented(indicator):
    function = getattr(tl, indicator.name)
    assert function.__doc__, f"{indicator.name} has no docstring"


def test_nothing_in_the_docs_reads_as_a_trade_instruction(indicator):
    """D11: outputs are measurements, not advice."""
    forbidden = (" buy ", " sell ", "entry point", "exit point", "recommendation")
    text = (getattr(tl, indicator.name).__doc__ or "").lower()
    for word in forbidden:
        assert word not in text, f"{indicator.name} docstring contains {word!r}"
