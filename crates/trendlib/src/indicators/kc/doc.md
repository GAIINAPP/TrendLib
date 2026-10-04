# Keltner Channel

An exponential average of the typical price with bands a number of average
true ranges above and below it. The width follows how far bars have been
travelling rather than how far they have been from their own mean, so a long
run of wide bars opens the channel even if the closes are orderly.

## Formula

$$
\text{middle}_t = \mathrm{EMA}\!\left(\tfrac{H + L + C}{3},\ n\right)_t \qquad
\text{upper}_t = \text{middle}_t + k\,\mathrm{ATR}_t \qquad
\text{lower}_t = \text{middle}_t - k\,\mathrm{ATR}_t
$$

where $n$ is `period`, $k$ is `nbdev` and the ATR is Wilder's over
`atr_period` bars.

## Conventions

- Warm-up is the larger of `period - 1` and `atr_period`. The true range
  reaches back to the previous close, so it needs one bar more than it
  averages.
- The range average is started so that its first value lands on the same bar
  as the middle band's rather than earlier. It is recursive, so starting it
  sooner would give a different number on every row afterwards, not just the
  early ones; that is why `kc`'s width is not `atr`'s on the same data.
- Recursive on both sides, so the result is path dependent
  (`CONVENTIONS.md` § 5); `unstable` applies.
- A negative `nbdev` puts each band on the other side of the middle; the
  outputs are not reordered.

## Example

```python
import trendlib as tl
upper, middle, lower = tl.kc(high, low, close, period=20, atr_period=10, nbdev=2.0)
```

## References

- Chester W. Keltner, *How to Make Money in Commodities*, Keltner Statistical Service, 1960.
