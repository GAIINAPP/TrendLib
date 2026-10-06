# Fibonacci Pivot Points

Floor-trader levels spaced by Fibonacci fractions: the previous bar's typical
price, and levels 0.382, 0.618, 1 and 1.382 of the previous bar's range either
side of it, as finta draws them.

## Formula

Row $t$ reads bar $t-1$ ($H$, $L$, $C$ below):

$$
\text{pp} = \frac{H + L + C}{3} \qquad
\text{r}_k = \text{pp} + f_k (H - L) \qquad \text{s}_k = \text{pp} - f_k (H - L)
$$

with $f_1, f_2, f_3, f_4 = 0.382, 0.618, 1, 1.382$.

## Conventions

- Outputs come in finta's order: pivot, the four supports, the four resistances.
- The fourth level (1.382) is finta's; TradingView's Fibonacci pivots stop at
  the third.
- Not path dependent: each row reads the bar before it and nothing else.
- Oracle F (`DECISIONS.md` D19): the golden files are `finta` 1.3's
  `TA.PIVOT_FIB`, test-only and never shipped.

## Example

```python
import trendlib as tl
lines = tl.pivots_fibonacci(high, low, close)
```

## References

- TradingView, Pivot Points Standard, type Fibonacci
- finta 1.3, finta.TA (oracle F)
