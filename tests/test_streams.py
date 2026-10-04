"""The streaming contract in docs/PYTHON_API.md section 4, for every indicator.

The promise that matters is the last one: for any history and any sequence of
bars, what `update` returns equals the batch function on the concatenated
series, bitwise. A dashboard and a backtest cannot disagree.
"""

import numpy as np
import pytest
from conftest import bitwise_equal

import trendlib as tl

pd = pytest.importorskip("pandas")


def as_list(value, indicator):
    return list(value) if indicator.multi_output else [value]


def same(left, right) -> bool:
    """Equality that treats NaN as equal to NaN.

    An indicator whose output is undefined for the test bars, such as an arc
    cosine of a price, answers NaN on every row. Plain `==` would call two
    identical streams different.
    """
    left = left if isinstance(left, tuple) else (left,)
    right = right if isinstance(right, tuple) else (right,)
    if len(left) != len(right):
        return False
    return all((a != a and b != b) or a == b for a, b in zip(left, right, strict=True))


def outputs_of(result, indicator):
    return list(result) if indicator.multi_output else [result]


def factory(indicator):
    return getattr(tl.stream, indicator.name)


def test_opening_with_exactly_the_lookback_says_how_many_bars_are_needed(bars, indicator):
    needed = indicator.lookback() + 1
    columns = indicator.columns(bars, needed - 1)
    with pytest.raises(tl.InsufficientHistory) as caught:
        factory(indicator)(*columns)
    assert f"at least {needed} valid bars" in str(caught.value)


def test_opening_with_one_more_bar_works(bars, indicator):
    needed = indicator.lookback() + 1
    handle = factory(indicator)(*indicator.columns(bars, needed))
    batch = outputs_of(indicator.call(bars, needed), indicator)
    assert handle.bars_seen == needed
    for value, column in zip(as_list(handle.value, indicator), batch, strict=True):
        assert value == pytest.approx(column[-1], nan_ok=True) or np.isnan(column[-1])


def test_update_reproduces_batch_bitwise(bars, indicator):
    rows = 400
    split = indicator.lookback() + 1
    columns = indicator.columns(bars, rows)
    handle = factory(indicator)(*indicator.columns(bars, split))

    streamed = [list(column) for column in outputs_of(indicator.call(bars, split), indicator)]
    for row in range(split, rows):
        bar = [column[row] for column in columns]
        values = as_list(handle.update(*bar), indicator)
        for held, value in zip(streamed, values, strict=True):
            held.append(value)

    expected = outputs_of(indicator.call(bars, rows), indicator)
    for held, want in zip(streamed, expected, strict=True):
        assert bitwise_equal(held, want)
    assert handle.bars_seen == rows


def test_open_and_fill_matches_batch_and_keeps_the_container(bars, indicator):
    rows = 200
    index = pd.date_range("2024-01-01", periods=rows, freq="D", tz="Asia/Kolkata")
    series = [pd.Series(column, index=index) for column in indicator.columns(bars, rows)]
    handle, filled = factory(indicator).open_and_fill(*series)
    expected = getattr(tl, indicator.name)(*series)

    if indicator.multi_output:
        assert isinstance(filled, pd.DataFrame)
        assert list(filled.columns) == indicator.outputs
        pd.testing.assert_frame_equal(filled, expected)
    else:
        assert isinstance(filled, pd.Series)
        assert filled.name == indicator.outputs[0]
        pd.testing.assert_series_equal(filled, expected)
    assert handle.bars_seen == rows


def test_peek_predicts_update_and_commits_nothing(bars, indicator):
    split = indicator.lookback() + 1
    columns = indicator.columns(bars, split + 25)
    handle = factory(indicator)(*indicator.columns(bars, split))

    for row in range(split, split + 25):
        bar = [column[row] for column in columns]
        before = handle.value
        first = handle.peek(*bar)
        second = handle.peek(*bar)
        assert same(first, second)
        assert same(handle.value, before)
        assert same(handle.update(*bar), first)


def test_copy_is_an_independent_fork(bars, indicator):
    rows = 200
    split = indicator.lookback() + 1
    columns = indicator.columns(bars, rows)
    handle = factory(indicator)(*indicator.columns(bars, split))
    fork = handle.copy()
    assert fork is not handle
    assert same(fork.value, handle.value)

    for row in range(split, split + 10):
        fork.update(*[column[row] * 1.5 + 3.0 for column in columns])

    expected = outputs_of(indicator.call(bars, rows), indicator)
    for row in range(split, rows):
        values = as_list(handle.update(*[column[row] for column in columns]), indicator)
        for value, want in zip(values, expected, strict=True):
            assert bitwise_equal([value], [want[row]])


@pytest.mark.parametrize("bad", [float("nan"), float("inf"), float("-inf")])
def test_a_bad_bar_is_rejected_and_the_stream_is_unchanged(bars, indicator, bad):
    split = indicator.lookback() + 1
    columns = indicator.columns(bars, split + 15)
    clean = factory(indicator)(*indicator.columns(bars, split))
    poisoned = factory(indicator)(*indicator.columns(bars, split))

    with pytest.raises(tl.InvalidInput):
        poisoned.update(*[bad] * len(indicator.inputs))
    with pytest.raises(tl.InvalidInput):
        poisoned.peek(*[bad] * len(indicator.inputs))
    assert poisoned.bars_seen == clean.bars_seen
    assert same(poisoned.value, clean.value)

    for row in range(split, split + 15):
        bar = [column[row] for column in columns]
        assert same(poisoned.update(*bar), clean.update(*bar))


def test_a_dataframe_history_is_accepted(bars, indicator):
    if sum(1 for name in indicator.inputs if name.startswith("source")) > 1:
        pytest.skip("a frame cannot say which column is which operand")
    rows = 200
    from_frame = factory(indicator)(indicator.frame(bars, rows))
    from_arrays = factory(indicator)(*indicator.columns(bars, rows))
    assert same(from_frame.value, from_arrays.value)


def test_parameters_are_fixed_for_the_life_of_the_stream(bars, indicator):
    handle = factory(indicator)(*indicator.columns(bars, 200))
    with pytest.raises(TypeError, match="unexpected parameter"):
        factory(indicator)(*indicator.columns(bars, 200), nonesuch=14)
    assert handle.bars_seen == 200


def test_repr_shows_the_name_and_position(bars, indicator):
    handle = factory(indicator)(*indicator.columns(bars, 200))
    text = repr(handle)
    assert indicator.name in text
    assert "bars_seen=200" in text


def test_leading_warm_up_rows_in_history_still_count_as_bars(bars, indicator):
    split = indicator.lookback() + 1
    history = [np.concatenate([[np.nan] * 3, column]) for column in indicator.columns(bars, split)]
    handle = factory(indicator)(*history)
    assert handle.bars_seen == split + 3
