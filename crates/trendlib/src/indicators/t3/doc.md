# Tillson T3

Six exponential averages chained together, then combined with weights that
cancel most of the lag the chaining introduced. `v_factor` sets how much of it
comes back out: at 0 the result is the sixth average itself, at 1 the
correction is at its strongest.

## Formula

With $e^{(1)} = \mathrm{EMA}(x, n)$ and $e^{(k)} = \mathrm{EMA}(e^{(k-1)}, n)$
for $k$ up to 6, and $v$ the `v_factor`,

$$
c_1 = -v^3 \quad c_2 = 3v^2 + 3v^3 \quad c_3 = -6v^2 - 3v - 3v^3 \quad c_4 = 1 + 3v + 3v^2 + v^3
$$

$$
\mathrm{t3}_t = c_1 e^{(6)}_t + c_2 e^{(5)}_t + c_3 e^{(4)}_t + c_4 e^{(3)}_t
$$

## Conventions

- Warm-up is `6 * (period - 1)` bars, one per stage.
- The four weights sum to 1 at every `v_factor`, so a series the stages leave
  unchanged comes through unchanged.
- With `period=1` every stage is the identity and so is the result.
- `v_factor` is between 0 and 1 inclusive; the oracle refuses anything outside
  that and so does this.
- Recursive, so the result is path dependent (`CONVENTIONS.md` § 5) and the
  early rows carry the seed; `unstable` applies.

## Example

```python
import trendlib as tl
smoothed = tl.t3(close, period=5, v_factor=0.7)
```

## References

- Tim Tillson, "Smoothing Techniques for More Accurate Signals", *Technical Analysis of Stocks and Commodities*, January 1998.
