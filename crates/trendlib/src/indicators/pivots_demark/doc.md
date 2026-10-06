# DeMark Pivot Points

Tom DeMark's levels: a sum of the previous bar that weights the low twice after
a falling bar, the high twice after a rising one and the close twice after a
flat one, and one level either side.

## Formula

Row $t$ reads bar $t-1$ ($O$, $H$, $L$, $C$ below):

$$
X = \begin{cases}
H + 2L + C & C < O\\
2H + L + C & C > O\\
H + L + 2C & C = O
\end{cases}
$$

$$
\text{pp} = \frac X4 \qquad r_1 = \frac X2 - L \qquad s_1 = \frac X2 - H
$$

## Conventions

- Warm-up is 1 bar: row `t` describes the range in force during period `t`,
  built from the bar before it.
- These are TradingView's "Pivot Points Standard" with type **DM**.
- Not path dependent: each row reads the bar before it and nothing else.
- Oracle A (`DECISIONS.md` D20): the golden files evaluate this formula through
  NumPy arithmetic; no TA-Lib function computes a step.

## Example

```python
import trendlib as tl
pp, r1, s1 = tl.pivots_demark(open, high, low, close)
```

## References

- Tom DeMark, as TradingView attributes the levels
- TradingView, Pivot Points Standard, type DM
