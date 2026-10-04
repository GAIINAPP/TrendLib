# Hull Moving Average

A weighted average built to follow turns quickly. Doubling a half-length
weighted average and subtracting the full-length one removes most of the lag,
and a short weighted average over that result smooths what the subtraction
leaves behind.

## Formula

$$
\mathrm{hma}_t = \mathrm{WMA}\Big(2\,\mathrm{WMA}(x, \lfloor n/2 \rfloor) - \mathrm{WMA}(x, n),\ \lfloor \sqrt{n} \rfloor\Big)_t
$$

where $x$ is `source` and $n$ is `period`.

## Conventions

- Warm-up is `(period - 1) + (floor(sqrt(period)) - 1)`: the full-length stage
  decides when the difference first exists, then the smoothing stage adds its
  own.
- Both the half length and the smoothing length are floored, so `period=20`
  uses 10 and 4.
- With `period=1` the half-length stage would have no bars to average, and the
  series is returned unchanged with a lookback of zero. That is what TA-Lib
  does.
- Not path dependent: every stage is a weighted mean over a window.

## Example

```python
import trendlib as tl
hull = tl.hma(close, period=20)
```

## References

- Alan Hull, "How to Reduce Lag in a Moving Average", 2005.
