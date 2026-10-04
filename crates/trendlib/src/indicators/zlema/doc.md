# Zero Lag Exponential Moving Average

An exponential average fed a series that has been pushed forward by roughly the
lag the average will introduce. The correction is the difference between the
current bar and one from half a period ago, added on top of the current bar.

## Formula

$$
\mathrm{zlema}_t = \mathrm{EMA}\big(2x_t - x_{t-\ell},\ n\big)
\qquad \ell = \left\lfloor \tfrac{n-1}{2} \right\rfloor
$$

where $x$ is `source` and $n$ is `period`.

## Conventions

- Warm-up is `floor((period - 1) / 2) + period - 1`: the correction needs a bar
  to reach back to before the average can start.
- At `period` 1 and 2 the lag rounds down to nothing and the correction is
  dropped, which leaves an ordinary exponential average.
- Doubling the current bar overshoots as well as removing lag, so the line can
  run past a turn before coming back. That is the trade the construction makes.
- Recursive, so the result is path dependent (`CONVENTIONS.md` § 5);
  `unstable` applies.

## Example

```python
import trendlib as tl
quick = tl.zlema(close, period=30)
```

## References

- John Ehlers and Ric Way, "Zero Lag (Well, Almost)", *Technical Analysis of Stocks and Commodities*, November 2010.
