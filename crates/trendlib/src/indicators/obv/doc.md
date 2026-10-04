# On Balance Volume

A running total of volume that adds the day's volume when the close is higher
than the one before and subtracts it when the close is lower. It asks whether
volume is arriving on up days or down days, and ignores how large the price
move was.

## Formula

$$
\mathrm{OBV}_t =
\begin{cases}
\mathrm{OBV}_{t-1} + v_t & c_t > c_{t-1}\\
\mathrm{OBV}_{t-1} - v_t & c_t < c_{t-1}\\
\mathrm{OBV}_{t-1} & c_t = c_{t-1}
\end{cases}
$$

with $\mathrm{OBV}$ seeded at the first bar's volume, where $c$ is `close` and
$v$ is `volume`.

## Conventions

- Lookback is 0: the first bar already has a value.
- **Path dependent.** The total carries every bar that came before it, so the
  same rows computed from a later starting point differ by a constant. A stream
  opened on history and fed later bars still reproduces the batch result for
  that same series exactly (`CONVENTIONS.md` section 5).
- An unchanged close leaves the total alone, whatever volume traded.
- Volume of zero is valid; a negative volume raises `InvalidInput`.
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

balance = tl.obv(df)
```

## References

- Joseph E. Granville, *Granville's New Key to Stock Market Profits*,
  Prentice-Hall, 1963.
