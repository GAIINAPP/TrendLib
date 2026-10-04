# Chaikin Accumulation Distribution Line

A running total of volume weighted by where each close finished inside its own
bar. A close at the high counts the whole bar's volume as accumulation, a close
at the low counts all of it the other way, and a close in the middle counts for
little either way.

## Formula

$$
m_t = \frac{(c_t - l_t) - (h_t - c_t)}{h_t - l_t} \qquad
\mathrm{AD}_t = \mathrm{AD}_{t-1} + m_t\,v_t
$$

where $h$, $l$, $c$ and $v$ are `high`, `low`, `close` and `volume`.

## Conventions

- Lookback is 0: the first bar already has a value.
- A bar whose high equals its low says nothing about where buyers or sellers
  won, so it contributes nothing instead of dividing by zero. TA-Lib skips such
  a bar too.
- **Path dependent**, like `obv`: the total carries everything before it
  (`CONVENTIONS.md` section 5).
- Volume of zero is valid; a negative volume raises `InvalidInput`.
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

line = tl.ad(df)
```

## References

- Marc Chaikin's accumulation/distribution line, as described in Steven B.
  Achelis, *Technical Analysis from A to Z*, McGraw-Hill, 2000.
