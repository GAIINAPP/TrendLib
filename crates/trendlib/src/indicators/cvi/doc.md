# Chaikin Volatility

How much wider or narrower the bars have become, as a percentage change in their smoothed range.

## Formula

$$
\mathrm{cvi}_t = 100 \cdot \frac{\mathrm{EMA}(H - L, n)_t - \mathrm{EMA}(H - L, n)_{t-m}}{\mathrm{EMA}(H - L, n)_{t-m}}
$$

where $n$ is `period` and $m$ is `roc_period`.

## Conventions

- Warm-up is `period - 1` plus `roc_period`.
- A smoothed range of exactly zero that far back reads a change of `0.0`, as
  TA-Lib does, rather than the `NaN` that `roc` would give. The test is exact.
- The range ignores gaps, so this measures how far bars travel within
  themselves rather than how far price moves between them.
- Recursive, so the result is path dependent (`CONVENTIONS.md` § 5);
  `unstable` applies.

## Example

```python
import trendlib as tl
cvi = tl.cvi(high, low)
```

## References

- Marc Chaikin, as described in Steven B. Achelis, Technical Analysis from A to Z, McGraw-Hill, 2000
