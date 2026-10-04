# Central Pivot Range

The pivot and the two lines around it that mark the day's expected range.

## Formula

From the previous bar's high, low and close,

$$
\mathrm{pivot} = \frac{H + L + C}{3} \qquad
\mathrm{bc} = \frac{H + L}{2} \qquad
\mathrm{tc} = 2\,\mathrm{pivot} - \mathrm{bc}
$$

## Conventions

- Warm-up is 1 bar: row `t` describes the range in force during period `t`,
  built from the bar before it. Nothing is reported from the current bar.
- Inputs are bars of the period the levels are built from, normally daily
  bars. Deriving a prior day from intraday bars is M6.
- Not path dependent: each row depends on one earlier bar and nothing else.
- `cpr_tc` is **below** `cpr_bc` when the previous close was below the midpoint
  of the range. The outputs are not reordered: the names say which formula
  produced them, not which is higher.
- A narrow range between the two is read as a quiet day ahead and a wide one as
  a busy one; the library reports the numbers, not the reading.

## Example

```python
import trendlib as tl
cpr_pivot, cpr_bc, cpr_tc = tl.cpr(high, low, close)
```

## References

- Frank Ochoa, Secrets of a Pivot Boss, Wiley, 2010
