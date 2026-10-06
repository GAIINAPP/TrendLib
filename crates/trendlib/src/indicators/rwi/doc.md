# Random Walk Index

Michael Poulos' Random Walk Index: how far price has risen from the low, and
fallen from the high, of `period` bars ago, in units of what a random walk with
the same average true range would travel.

## Formula

$$
\text{rwi\_high}_t = \frac{H_t - L_{t-n}}{\mathrm{ATR}_n(t)\,\sqrt n} \qquad
\text{rwi\_low}_t = \frac{H_{t-n} - L_t}{\mathrm{ATR}_n(t)\,\sqrt n}
$$

where $n$ is `period` and $\mathrm{ATR}_n$ is TrendLib's `atr`.

## Conventions

- One look-back length, `period`, as TradingView's Random Walk Index uses.
  Poulos also describes taking the largest reading over a range of lengths; that
  is not computed here.
- Where the average true range is 0 nothing has moved, and both lines read 0.
- Warm-up is `period` bars.
- Recursive through the average true range's Wilder smoothing.
- Oracle A (`DECISIONS.md` D20): the golden files evaluate this formula through
  TA-Lib's `ATR`, `SUB` and `MULT`, the division in NumPy.

## Example

```python
import trendlib as tl
high, low = tl.rwi(high, low, close)
```

## References

- Michael Poulos, Of Trends and Random Walks, Technical Analysis of Stocks & Commodities, February 1991
- TradingView, Random Walk Index
