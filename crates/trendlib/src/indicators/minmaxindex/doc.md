# Indices of the Rolling Minimum and Maximum

Which bars hold the lowest and the highest value of the last `period` bars,
reported together as rows of the series that was passed in.

## Formula

$$
\mathrm{min\_index}_t = \arg\min_{i=t-n+1}^{t} x_i \qquad
\mathrm{max\_index}_t = \arg\max_{i=t-n+1}^{t} x_i
$$

where $x$ is `source` and $n$ is `period`.

## Conventions

- Lookback is $n - 1$.
- The index counts rows of the array that was passed in, not rows of the
  window, so leading warm-up rows move it by their own length. The spec carries
  the `absolute_index` flag and the suites check that it shifts.
- Ties keep the earliest bar, so on a flat stretch the index crawls forward
  only as the window slides past it. `aroon` keeps the latest instead, which
  is what TA-Lib does for each.
- Warm-up rows are `0`, as integer outputs are (`CONVENTIONS.md` section 2).
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

low_at, high_at = tl.minmaxindex(close, period=30)
```

## References

- Steven B. Achelis, Technical Analysis from A to Z, McGraw-Hill, 2000
