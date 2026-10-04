# Camarilla Pivot Points

Four levels either side of the previous close, spaced by fractions of its range.

## Formula

From the previous bar's close and range $R = H - L$,

$$
r_k = C + \frac{1.1\,R}{d_k} \qquad s_k = C - \frac{1.1\,R}{d_k}
$$

with $d_1 = 12$, $d_2 = 6$, $d_3 = 4$ and $d_4 = 2$.

## Conventions

- Warm-up is 1 bar: row `t` describes the range in force during period `t`,
  built from the bar before it. Nothing is reported from the current bar.
- Inputs are bars of the period the levels are built from, normally daily
  bars. Deriving a prior day from intraday bars is M6.
- Not path dependent: each row depends on one earlier bar and nothing else.
- The levels are spaced around the previous **close**, not around a pivot, so
  a bar that closed near one end of its range gives an asymmetric picture of
  where the range sat.
- These are TradingView's "Pivot Points Standard" with type **Camarilla**,
  levels 1 to 4.

## Example

```python
import trendlib as tl
camarilla_r1, camarilla_r2, camarilla_r3, camarilla_r4, camarilla_s1, camarilla_s2, camarilla_s3, camarilla_s4 = tl.pivots_camarilla(high, low, close)
```

## References

- Nick Scott's Camarilla equation, 1989
