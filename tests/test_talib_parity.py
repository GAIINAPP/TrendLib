"""Agreement with the oracle, ta-lib-python.

The golden files pin fixed cases; this suite throws random data and random
parameters at both implementations. Tolerance is the one docs/SPEC.md section 6
requires: 1e-10 relative, 1e-12 absolute near zero.

Where TrendLib happens to reproduce TA-Lib bit for bit, that is asserted too,
because `doc.md` claims it and an unasserted claim rots.
"""

import numpy as np
import pytest
from hypothesis import HealthCheck, given, settings
from hypothesis import strategies as st
from hypothesis.extra.numpy import arrays

import trendlib as tl

talib = pytest.importorskip("talib", reason="the TA-Lib oracle is not installed here")

from talib import abstract  # noqa: E402

pytestmark = pytest.mark.talib

RTOL = 1e-10
ATOL = 1e-12

ALIASES = {"sma": "SMA", "ema": "EMA", "rsi": "RSI"}
MIN_PERIOD = {"sma": 1, "ema": 1, "rsi": 2}
# SMA is pure addition, subtraction and one division, so there is no multiply-add
# for a compiler to contract and TrendLib reproduces TA-Lib exactly. EMA and RSI
# cannot: the published TA-Lib wheel evaluates the EMA step as a single fused
# multiply-add, and its Wilder step rounds in an order the formula does not fix.
# Both stay far inside the tolerance docs/SPEC.md section 6 requires.
BITWISE = {"sma"}

# Subnormals are excluded on purpose. Below about 2.2e-308 a float64 carries
# fewer significand bits with every halving, so two correct implementations can
# drift far past 1e-10 from each other on the same input; the accuracy contract
# in docs/SPEC.md section 6 covers normal float64. `test_subnormal_input_is_defined`
# pins the behaviour that is still promised there.
prices = arrays(
    dtype=np.float64,
    shape=st.integers(min_value=0, max_value=400),
    elements=st.floats(
        min_value=-1.0e6,
        max_value=1.0e6,
        allow_nan=False,
        allow_infinity=False,
        allow_subnormal=False,
        width=64,
    ),
)


def assert_matches(name, mine, theirs):
    assert mine.shape == theirs.shape
    np.testing.assert_array_equal(
        np.isnan(mine), np.isnan(theirs), err_msg=f"{name}: NaN rows differ"
    )
    defined = ~np.isnan(theirs)
    np.testing.assert_allclose(mine[defined], theirs[defined], rtol=RTOL, atol=ATOL, err_msg=name)
    if name in BITWISE:
        assert np.array_equal(mine[defined].view(np.uint64), theirs[defined].view(np.uint64)), (
            f"{name}: documented as bitwise identical to TA-Lib, but it is not"
        )


@settings(max_examples=250, deadline=None, suppress_health_check=[HealthCheck.too_slow])
@given(series=prices, seed=st.integers(min_value=0, max_value=1000))
@pytest.mark.parametrize("name", ["sma", "ema", "rsi"])
def test_batch_agrees_with_talib(name, series, seed):
    period = MIN_PERIOD[name] + seed % (60 - MIN_PERIOD[name] + 1)
    mine = getattr(tl, name)(series, period=period)
    theirs = getattr(talib, ALIASES[name])(series, period)
    assert_matches(name, mine, theirs)


@pytest.mark.parametrize("name", ["sma", "ema", "rsi"])
def test_batch_agrees_with_talib_on_the_committed_dataset(name, closes):
    for period in (MIN_PERIOD[name], 2, 3, 14, 30, 200, 999):
        mine = getattr(tl, name)(closes, period=period)
        theirs = getattr(talib, ALIASES[name])(closes, period)
        assert_matches(name, mine, theirs)


@pytest.mark.parametrize("name", ["sma", "ema", "rsi"])
def test_lookback_equals_talibs(name):
    function = abstract.Function(ALIASES[name])
    for period in (MIN_PERIOD[name], 2, 5, 14, 30, 100, 100000):
        function.parameters = {"timeperiod": period}
        assert tl.lookback(name, period=period) == function.lookback, (name, period)


@pytest.mark.parametrize("name", ["sma", "ema", "rsi"])
def test_a_constant_series_matches_talib(name):
    flat = np.full(120, 7.5)
    period = 14 if name == "rsi" else 30
    assert_matches(
        name, getattr(tl, name)(flat, period=period), getattr(talib, ALIASES[name])(flat, period)
    )


@pytest.mark.parametrize("name", ["sma", "ema", "rsi"])
@pytest.mark.parametrize("scale", [1.0e-300, 1.0e-150, 1.0e150, 1.0e300])
def test_extreme_magnitudes_match_talib(name, scale):
    series = scale * np.array([1.0 + (i % 7) / 10.0 for i in range(120)])
    period = 14 if name == "rsi" else 20
    assert_matches(
        name,
        getattr(tl, name)(series, period=period),
        getattr(talib, ALIASES[name])(series, period),
    )


@pytest.mark.parametrize("name", ["sma", "ema", "rsi"])
def test_leading_nan_rows_match_talib(name, closes):
    padded = np.concatenate([[np.nan] * 6, closes[:300]])
    period = 14 if name == "rsi" else 20
    assert_matches(
        name,
        getattr(tl, name)(padded, period=period),
        getattr(talib, ALIASES[name])(padded, period),
    )


@pytest.mark.parametrize("name", ["sma", "ema", "rsi"])
def test_the_uppercase_alias_computes_the_same_thing(name, closes):
    lower = getattr(tl, name)(closes, period=9)
    upper = getattr(tl, ALIASES[name])(closes, timeperiod=9)
    assert np.array_equal(lower.view(np.uint64), upper.view(np.uint64))


@pytest.mark.parametrize("name", ["sma", "ema", "rsi"])
def test_the_alias_defaults_match_talibs(name):
    function = abstract.Function(ALIASES[name])
    talib_default = function.info["parameters"]["timeperiod"]
    assert tl.lookback(name) == tl.lookback(name, period=talib_default)


@pytest.mark.parametrize("period", [2, 14, 30, 200, 999])
def test_the_ema_gap_to_talib_is_a_rounding_step(closes, period):
    """TA-Lib's wheel fuses the EMA update; TrendLib rounds twice.

    `prev + (x - prev) * k` compiled with a contracted multiply-add rounds once
    where TrendLib rounds the multiply and the add separately, so the two can
    differ by one unit in the last place and then track each other. On price
    data that stays a unit or two; on a series that decays towards zero for
    hundreds of bars the gap compounds, which is why the contract in
    docs/SPEC.md section 6 is a relative tolerance and not a ULP count.

    It is recorded rather than fixed: `f64::mul_add` would match the wheel on
    hardware that has an FMA and fall back to a software implementation where
    it does not (DECISIONS.md A11).
    """
    mine, theirs = tl.ema(closes, period=period), talib.EMA(closes, period)
    defined = ~np.isnan(theirs)
    gap = np.abs(
        mine[defined].view(np.int64).astype(object) - theirs[defined].view(np.int64).astype(object)
    )
    assert gap.max() <= 4, f"period={period}: {gap.max()} ULP"
    assert np.max(np.abs(mine[defined] - theirs[defined]) / np.abs(theirs[defined])) < 1e-14


@pytest.mark.parametrize("name", ["sma", "ema", "rsi"])
def test_subnormal_input_is_defined_even_though_the_tolerance_does_not_apply(name):
    """Behaviour is defined on subnormal input; agreement with the oracle is not.

    docs/TESTING.md section 5 asks for defined behaviour on extreme magnitudes.
    What is promised is a full-length output, warm-up rows in the documented
    places and no panic. Agreement with TA-Lib is not promised: every arithmetic
    step on a subnormal loses significand bits, so two correct implementations
    diverge without either being wrong.
    """
    series = np.full(60, 0.0)
    series[10] = 5e-310  # subnormal: below the smallest normal float64
    out = getattr(tl, name)(series, period=10)
    lookback = tl.lookback(name, period=10)
    assert out.shape == series.shape
    assert np.isnan(out[:lookback]).all()
    assert np.isfinite(out[lookback:]).all()
