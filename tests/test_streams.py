"""The streaming contract in docs/PYTHON_API.md section 4.

The promise that matters is the last one: for any history and any sequence of
bars, the values `update` returns equal the batch function's values on the
concatenated series, bitwise. A dashboard and a backtest cannot disagree.
"""

import numpy as np
import pytest
from conftest import bitwise_equal

import trendlib as tl

pd = pytest.importorskip("pandas")


def split_for(name, period):
    return tl.lookback(name, period=period) + 1


def test_opening_with_exactly_the_lookback_says_how_many_bars_are_needed(closes, indicator):
    period = 10
    needed = split_for(indicator, period)
    with pytest.raises(tl.InsufficientHistory) as caught:
        getattr(tl.stream, indicator)(closes[: needed - 1], period=period)
    assert f"at least {needed} valid bars" in str(caught.value)
    assert issubclass(tl.InsufficientHistory, ValueError)


def test_opening_with_one_more_bar_works(closes, indicator):
    period = 10
    needed = split_for(indicator, period)
    handle = getattr(tl.stream, indicator)(closes[:needed], period=period)
    batch = getattr(tl, indicator)(closes[:needed], period=period)
    assert handle.value == batch[-1]
    assert handle.bars_seen == needed


def test_update_reproduces_batch_bitwise(closes, indicator):
    period = 14
    split = split_for(indicator, period)
    handle = getattr(tl.stream, indicator)(closes[:split], period=period)
    streamed = list(getattr(tl, indicator)(closes[:split], period=period))
    for bar in closes[split:]:
        streamed.append(handle.update(bar))
    assert bitwise_equal(streamed, getattr(tl, indicator)(closes, period=period))
    assert handle.bars_seen == len(closes)


def test_open_and_fill_matches_batch_and_keeps_the_container(closes, indicator):
    period = 14
    index = pd.date_range("2024-01-01", periods=len(closes), freq="D", tz="Asia/Kolkata")
    series = pd.Series(closes, index=index)
    handle, filled = getattr(tl.stream, indicator).open_and_fill(series, period=period)
    expected = getattr(tl, indicator)(series, period=period)
    assert isinstance(filled, pd.Series)
    assert filled.name == indicator
    assert filled.index.equals(index)
    assert bitwise_equal(filled.to_numpy(), expected.to_numpy())
    assert handle.value == expected.to_numpy()[-1]


def test_peek_predicts_update_and_commits_nothing(closes, indicator):
    period = 14
    split = split_for(indicator, period)
    handle = getattr(tl.stream, indicator)(closes[:split], period=period)
    for bar in closes[split : split + 25]:
        before = handle.value
        first = handle.peek(bar)
        second = handle.peek(bar)
        assert first == second
        assert handle.value == before
        assert handle.update(bar) == first


def test_copy_is_an_independent_fork(closes, indicator):
    period = 14
    split = split_for(indicator, period)
    handle = getattr(tl.stream, indicator)(closes[:split], period=period)
    fork = handle.copy()
    assert fork is not handle
    assert fork.value == handle.value

    for bar in closes[split : split + 10]:
        fork.update(bar * 1.5 + 3.0)

    expected = getattr(tl, indicator)(closes, period=period)
    streamed = [handle.update(bar) for bar in closes[split:]]
    assert bitwise_equal(streamed, expected[split:])


@pytest.mark.parametrize("bad", [float("nan"), float("inf"), float("-inf")])
def test_a_bad_bar_is_rejected_and_the_stream_is_unchanged(closes, indicator, bad):
    period = 14
    split = split_for(indicator, period)
    clean = getattr(tl.stream, indicator)(closes[:split], period=period)
    poisoned = getattr(tl.stream, indicator)(closes[:split], period=period)

    with pytest.raises(tl.InvalidInput):
        poisoned.update(bad)
    with pytest.raises(tl.InvalidInput):
        poisoned.peek(bad)
    assert poisoned.bars_seen == clean.bars_seen
    assert poisoned.value == clean.value

    for bar in closes[split : split + 15]:
        assert poisoned.update(bar) == clean.update(bar)


def test_a_dataframe_history_uses_its_close_column(closes, indicator):
    frame = pd.DataFrame({"open": closes, "high": closes, "low": closes, "close": closes})
    from_frame = getattr(tl.stream, indicator)(frame, period=14)
    from_array = getattr(tl.stream, indicator)(closes, period=14)
    assert from_frame.value == from_array.value


def test_parameters_are_fixed_for_the_life_of_the_stream(closes, indicator):
    handle = getattr(tl.stream, indicator)(closes[:100], period=14)
    assert not hasattr(handle, "period")
    with pytest.raises(TypeError, match="unexpected parameter"):
        getattr(tl.stream, indicator)(closes[:100], timeperiod=14)


def test_an_out_of_range_period_is_reported_the_same_way(closes):
    with pytest.raises(tl.InvalidInput) as caught:
        tl.stream.rsi(closes[:100], period=1)
    assert str(caught.value) == "rsi: period=1 is out of range [2, 100000]"


def test_repr_shows_the_name_and_position(closes, indicator):
    handle = getattr(tl.stream, indicator)(closes[:100], period=14)
    text = repr(handle)
    assert indicator in text
    assert "bars_seen=100" in text


def test_leading_warm_up_rows_in_history_still_count_as_bars(closes, indicator):
    period = 14
    split = split_for(indicator, period)
    history = np.concatenate([[np.nan] * 3, closes[:split]])
    handle = getattr(tl.stream, indicator)(history, period=period)
    assert handle.bars_seen == len(history)
    assert handle.value == getattr(tl, indicator)(history, period=period)[-1]
