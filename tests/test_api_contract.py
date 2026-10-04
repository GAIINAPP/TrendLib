"""Every rule in docs/PYTHON_API.md that M1 is supposed to satisfy."""

import numpy as np
import pytest

import trendlib as tl

pd = pytest.importorskip("pandas")


def test_numpy_in_numpy_out(closes, indicator):
    out = getattr(tl, indicator)(closes)
    assert isinstance(out, np.ndarray)
    assert out.dtype == np.float64
    assert out.shape == closes.shape


def test_list_in_numpy_out():
    out = tl.sma([1.0, 2.0, 3.0, 4.0], period=2)
    assert isinstance(out, np.ndarray)
    assert out.shape == (4,)


def test_pandas_series_keeps_its_index_and_is_named_after_the_output(closes, indicator):
    index = pd.date_range("2024-01-01", periods=len(closes), freq="D", tz="Asia/Kolkata")
    series = pd.Series(closes, index=index)
    out = getattr(tl, indicator)(series)
    assert isinstance(out, pd.Series)
    assert out.name == indicator
    assert out.index.equals(index)
    assert out.dtype == np.float64


def test_a_dataframe_uses_its_close_column(closes, indicator):
    frame = pd.DataFrame({"open": closes, "high": closes, "low": closes, "close": closes})
    from_frame = getattr(tl, indicator)(frame)
    from_series = getattr(tl, indicator)(frame["close"])
    assert isinstance(from_frame, pd.Series)
    pd.testing.assert_series_equal(from_frame, from_series)


def test_column_matching_ignores_case(closes):
    frame = pd.DataFrame({"Close": closes})
    assert tl.sma(frame).notna().any()


def test_a_frame_without_close_names_the_missing_column(closes):
    frame = pd.DataFrame({"price": closes})
    with pytest.raises(tl.InvalidInput, match="no 'close' column"):
        tl.sma(frame)


def test_parameters_are_keyword_only(closes, indicator):
    with pytest.raises(TypeError):
        getattr(tl, indicator)(closes, 10)


def test_an_out_of_range_period_names_the_range(closes):
    with pytest.raises(tl.InvalidInput) as caught:
        tl.rsi(closes, period=1)
    assert str(caught.value) == "rsi: period=1 is out of range [2, 100000]"

    with pytest.raises(tl.InvalidInput) as caught:
        tl.sma(closes, period=0)
    assert str(caught.value) == "sma: period=0 is out of range [1, 100000]"


def test_a_negative_period_reports_the_value_the_caller_passed(closes):
    with pytest.raises(tl.InvalidInput) as caught:
        tl.ema(closes, period=-5)
    assert str(caught.value) == "ema: period=-5 is out of range [1, 100000]"


def test_a_non_integer_period_is_rejected(closes):
    with pytest.raises(tl.InvalidInput, match="period must be an integer"):
        tl.rsi(closes, period=14.5)


def test_errors_are_value_errors(closes):
    assert issubclass(tl.InvalidInput, ValueError)
    with pytest.raises(ValueError):
        tl.rsi(closes, period=1)


def test_a_nan_after_the_first_valid_bar_names_the_input_and_row(closes):
    series = closes[:100].copy()
    series[42] = np.nan
    with pytest.raises(tl.InvalidInput) as caught:
        tl.sma(series, period=5)
    assert "source" in str(caught.value)
    assert "row 42" in str(caught.value)


def test_leading_nan_rows_are_skipped(closes, indicator):
    trimmed = getattr(tl, indicator)(closes)
    padded = getattr(tl, indicator)(np.concatenate([[np.nan] * 4, closes]))
    assert np.array_equal(padded[:4], padded[:4] * np.nan, equal_nan=True)
    assert np.array_equal(padded[4:], trimmed, equal_nan=True)


def test_empty_input_returns_empty_output(indicator):
    out = getattr(tl, indicator)(np.array([], dtype=np.float64))
    assert isinstance(out, np.ndarray)
    assert out.shape == (0,)


def test_short_input_is_all_warm_up(indicator):
    period = 5
    lookback = tl.lookback(indicator, period=period)
    out = getattr(tl, indicator)(np.arange(lookback, dtype=np.float64), period=period)
    assert np.isnan(out).all()


def test_the_input_is_never_modified(closes, indicator):
    original = closes.copy()
    getattr(tl, indicator)(closes)
    assert np.array_equal(closes, original)


def test_the_output_is_a_new_array(closes):
    first = tl.sma(closes)
    second = tl.sma(closes)
    assert first is not second
    assert not np.shares_memory(first, closes)


def test_a_non_contiguous_input_is_handled(closes, indicator):
    strided = closes[::2]
    assert not strided.flags["C_CONTIGUOUS"]
    out = getattr(tl, indicator)(strided)
    assert out.shape == strided.shape


def test_an_integer_input_is_accepted():
    out = tl.sma(np.arange(10, dtype=np.int64), period=3)
    assert out.dtype == np.float64


def test_a_two_dimensional_input_is_rejected():
    with pytest.raises(tl.InvalidInput, match="one-dimensional"):
        tl.sma(np.zeros((4, 4)), period=2)


def test_lookback(closes):
    assert tl.lookback("sma", period=30) == 29
    assert tl.lookback("ema", period=30) == 29
    assert tl.lookback("rsi", period=14) == 14
    assert tl.lookback("rsi") == 14


def test_lookback_rejects_an_unknown_name():
    with pytest.raises(tl.InvalidInput, match="no indicator named"):
        tl.lookback("nonesuch", period=5)


def test_lookback_rejects_an_unknown_parameter():
    with pytest.raises(TypeError, match="no parameter"):
        tl.lookback("rsi", timeperiod=14)


def test_public_surface_matches_the_contract():
    """Errors, versions, the stream namespace, and every generated function."""
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
    assert not any(name.startswith("_") and not name.startswith("__") for name in tl.__all__)


def test_nothing_in_the_docs_reads_as_a_trade_instruction():
    """D11: outputs are measurements, not advice."""
    forbidden = (" buy ", " sell ", "entry", "exit point", "recommendation", "target price")
    for name in ("sma", "ema", "rsi", "SMA", "EMA", "RSI", "lookback"):
        text = (getattr(tl, name).__doc__ or "").lower()
        for word in forbidden:
            assert word not in text, f"{name} docstring contains {word!r}"
