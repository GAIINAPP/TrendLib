# Average Directional Movement Index Rating

The average directional index smoothed once more, by averaging it with its own
reading from earlier in the window. It lags the index it is built from and moves
more slowly.

## Formula

$$
\mathrm{ADXR}_t = \frac{\mathrm{ADX}_t + \mathrm{ADX}_{t-(n-1)}}{2}
$$

## Conventions

- Lookback is $3n - 2$: the average index needs $2n - 1$ bars and the
  comparison then reaches $n - 1$ further back.
- Recursive, so early rows still carry a trace of the seed (D9).
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

strength = tl.adxr(df, period=14)
```

## References

- J. Welles Wilder Jr., New Concepts in Technical Trading Systems, Trend Research, 1978
