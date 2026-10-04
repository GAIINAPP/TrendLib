# Average Directional Movement Index

A smoothed reading of how one-sided movement has been. It measures how decisive
recent movement was rather than its direction, so it rises in a move either way
and falls when the two directions are evenly matched.

## Formula

$$
\mathrm{ADX}_n = \frac{1}{n}\sum \mathrm{DX} \qquad \mathrm{ADX}_t = \frac{(n-1)\,\mathrm{ADX}_{t-1}}{n} + \frac{\mathrm{DX}_t}{n}
$$

## Conventions

- Lookback is $2n - 1$: the index itself needs $n$ bars, and the average then
  needs $n$ of those.
- A Wilder average here, not the running total the components use, seeded with
  the mean of the first $n$ readings.
- Recursive, so early rows still carry a trace of the seed (D9).
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

strength = tl.adx(df, period=14)
```

## References

- J. Welles Wilder Jr., New Concepts in Technical Trading Systems, Trend Research, 1978
