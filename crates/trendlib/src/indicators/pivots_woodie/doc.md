# Woodie Pivot Points

Ken Wood's floor-trader levels: a pivot that weights the previous close twice,
with two levels either side of it.

## Formula

Row $t$ reads bar $t-1$ ($H$, $L$, $C$ below):

$$
\text{pp} = \frac{H + L + 2C}{4}
$$

$$
\begin{aligned}
r_1 &= 2\,\text{pp} - L & s_1 &= 2\,\text{pp} - H\\
r_2 &= \text{pp} + (H - L) & s_2 &= \text{pp} - (H - L)
\end{aligned}
$$

## Conventions

- Warm-up is 1 bar: row `t` describes the range in force during period `t`,
  built from the bar before it, as `pivots_traditional` does.
- The pivot is Wood's, from the previous close. TradingView's Woodie type uses
  the current period's open in its place; that needs an input this function does
  not take.
- Not path dependent: each row reads the bar before it and nothing else.
- Oracle A (`DECISIONS.md` D20): the golden files evaluate this formula through
  TA-Lib's `ADD`, `SUB`, `MULT` and `DIV`.

## Example

```python
import trendlib as tl
lines = tl.pivots_woodie(high, low, close)
```

## References

- Ken Wood, Woodies CCI Club
- TradingView, Pivot Points Standard, type Woodie
