# Stochastic Momentum Index

Where the close sits relative to the middle of the recent range rather than to its bottom, smoothed twice.

## Formula

With $\mathrm{mid}_t = \frac{1}{2}\left(\max H + \min L\right)$ over `period`
bars and $d_t = C_t - \mathrm{mid}_t$,

$$
\mathrm{smi}_t = 100 \cdot
\frac{\mathrm{EMA}(\mathrm{EMA}(d, n_s), n_f)_t}
{\tfrac{1}{2}\mathrm{EMA}(\mathrm{EMA}(\max H - \min L, n_s), n_f)_t}
$$

## Conventions

- Warm-up is `period - 1` plus all three averages' own.
- Measuring from the middle of the range rather than from its low is what
  separates this from `stoch`: the result runs from -100 to 100 rather than 0
  to 100.
- A range of exactly zero reads `0.0`, as TA-Lib does: the close has nowhere
  to sit. The test is exact.
- Both columns start on the same row.
- Recursive, so the result is path dependent (`CONVENTIONS.md` § 5).

## Example

```python
import trendlib as tl
smi, smi_smisignal = tl.smi(high, low, close)
```

## References

- William Blau, Momentum, Direction and Divergence, Wiley, 1995
