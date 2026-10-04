# True Strength Index

How much of the recent movement went one way, after smoothing both the movement and its size twice.

## Formula

With $d_t = x_t - x_{t-1}$,

$$
\mathrm{tsi}_t = 100 \cdot
\frac{\mathrm{EMA}(\mathrm{EMA}(d, n_1), n_2)_t}{\mathrm{EMA}(\mathrm{EMA}(|d|, n_1), n_2)_t}
$$

## Conventions

- Warm-up is `first_period + second_period - 1`: one bar for the change and
  then the two stages.
- A series that never moves reads `0.0`, as TA-Lib does. The test is exact.
- Smoothing the signed and the absolute movement the same way is what bounds
  the result to ±100.
- Recursive, so the result is path dependent (`CONVENTIONS.md` § 5).

## Example

```python
import trendlib as tl
tsi = tl.tsi(source)
```

## References

- William Blau, Momentum, Direction and Divergence, Wiley, 1995
