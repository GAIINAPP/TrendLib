# Directional Movement Index

How lopsided recent movement has been, whichever way it went. It reads 0 when
the two directions are matched and 100 when all the movement was one way, and
says nothing about which way that was.

## Formula

$$
\mathrm{DX}_t = 100\,\frac{\bigl|\mathrm{+DI}_t - \mathrm{-DI}_t\bigr|}{\mathrm{+DI}_t + \mathrm{-DI}_t}
$$

## Conventions

- Lookback is $n$, the same as the two indicators it compares. The movement and range totals exist a bar earlier, but the
  ratio is not reported until one bar after that, which is where TA-Lib starts
  it.
- Both totals are seeded over $n - 1$ bars and decayed by $n$, Wilder's running
  total rather than an average. The scale cancels in the ratio.
- A window with no true range at all reports 0 rather than dividing by zero.
- Recursive, so early rows still carry a trace of the seed (D9).
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).
- When both indicators are zero there is no spread to report and the value is 0.

## Example

```python
import trendlib as tl

strength = tl.dx(df, period=14)
```

## References

- J. Welles Wilder Jr., New Concepts in Technical Trading Systems, Trend Research, 1978
