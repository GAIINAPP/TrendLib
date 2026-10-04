# Average True Range

A smoothed measure of how much ground price covers per bar, gaps included. It is
Wilder's average of the true range, so it describes the size of recent moves
rather than their direction, and it is quoted in the same units as price.

## Formula

$$
\mathrm{ATR}_n = \frac{1}{n}\sum_{i=1}^{n}\mathrm{TR}_i \qquad
\mathrm{ATR}_t = \frac{(n-1)\,\mathrm{ATR}_{t-1}}{n} + \frac{\mathrm{TR}_t}{n},\quad t > n
$$

where $\mathrm{TR}$ is the true range and $n$ is `period`.

## Conventions

- Lookback is $n$, not $n - 1$: one bar is consumed before the first true range
  exists, and the average then needs $n$ of them. This equals TA-Lib's lookback,
  which the tests assert.
- `period = 1` leaves the true range unsmoothed, so `atr` equals `trange`.
- Wilder smoothing, seeded with the simple average of the first $n$ true ranges
  (`CONVENTIONS.md` section 4). Recursive, so early rows still carry a trace of
  the seed; there is no unstable-period setting (D9).
- Leading `NaN` rows are skipped and the seed starts at the first valid bar; a
  `NaN` or infinity after it raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

span = tl.atr(df, period=14)
```

## References

- J. Welles Wilder Jr., *New Concepts in Technical Trading Systems*, Trend
  Research, 1978, chapter 2.
