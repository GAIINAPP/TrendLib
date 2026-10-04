# Minus Directional Indicator

How much of the ground price has covered recently was downward movement, as a
percentage. It measures direction without saying anything about how far price
actually travelled.

## Formula

$$
\mathrm{-DI}_t = 100\,\frac{\mathrm{-DM}_t}{\mathrm{TR}_t}
$$

## Conventions

- Lookback is $n$. The movement and range totals exist a bar earlier, but the
  ratio is not reported until one bar after that, which is where TA-Lib starts
  it.
- Both totals are seeded over $n - 1$ bars and decayed by $n$, Wilder's running
  total rather than an average. The scale cancels in the ratio.
- A window with no true range at all reports 0 rather than dividing by zero.
- `period = 1` is still scaled to a percentage. TA-Lib returns the raw
  fraction there, a factor of exactly 100 smaller, while agreeing exactly at
  every other period; this is Deviation 7 in `CONVENTIONS.md` section 9.
- Recursive, so early rows still carry a trace of the seed (D9).
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

strength = tl.minus_di(df, period=14)
```

## References

- J. Welles Wilder Jr., New Concepts in Technical Trading Systems, Trend Research, 1978
