# Guppy Multiple Moving Average

Daryl Guppy's twelve exponential moving averages: six short ones (3 to 15 bars)
and six long ones (30 to 60). How tightly each group bunches, and whether the
groups cross, is how the chart is commonly read.

## Formula

For each $n$ in $3, 5, 8, 10, 12, 15$ (the short group) and $30, 35, 40, 45,
50, 60$ (the long group),

$$
E^{(n)}_{n-1} = \frac1n \sum_{j=0}^{n-1} x_j \qquad
E^{(n)}_t = E^{(n)}_{t-1} + \frac{2}{n+1}\big(x_t - E^{(n)}_{t-1}\big)
$$

where $x$ is `source`; each is TrendLib's `ema` at that period.

## Conventions

- The twelve periods are Guppy's and are not parameters.
- Every output starts on the same row, the 60-bar average's: 59.
- Recursive: each average still carries a trace of its seed in early rows.
- Oracle A (`DECISIONS.md` D20): the golden files evaluate this formula through
  TA-Lib's `EMA`.

## Example

```python
import trendlib as tl
lines = tl.guppy(source)
```

## References

- Daryl Guppy, Trading Tactics, Wrightbooks, 1997
