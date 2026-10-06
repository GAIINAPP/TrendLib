# Swing Index

J. Welles Wilder's Swing Index: one bar's change in close, close against open
and the previous bar's body, weighted by how far the bar reached from the
previous close.

## Formula

For bar $t \ge 1$, with $O, H, L, C$ its prices and $O', C'$ the previous bar's,
let $a = |H - C'|$, $b = |L - C'|$, $s = |H - L|$, $q = |C' - O'|$ and

$$
R = \begin{cases}
a - \tfrac12 b + \tfrac14 q & a \text{ is the largest of } a, b, s\\
b - \tfrac12 a + \tfrac14 q & b \text{ is the largest}\\
s + \tfrac14 q & \text{otherwise}
\end{cases}
\qquad K = \max(a, b)
$$

$$
\mathrm{SI}_t = 50\,\frac{(C - C') + \tfrac12 (C - O) + \tfrac14 (C' - O')}{R}\,\frac{K}{T}
$$

where $T$ is `limit_move`, and $\mathrm{SI}_t = 0$ when $R = 0$.

## Conventions

- `limit_move` is the largest move allowed in one bar, in price units, which
  futures exchanges set and stock markets mostly do not. It scales every value;
  with 0 the output is infinite.
- A bar with $R = 0$, where neither it nor the bar before it moved, reads 0.
- Warm-up is 1 bar.
- Not path dependent: each row reads its bar and the one before.
- Oracle A (`DECISIONS.md` D20): the golden files evaluate this formula through
  NumPy arithmetic alone; no TA-Lib function computes a step of it.

## Example

```python
import trendlib as tl
swing_index = tl.swing_index(open, high, low, close)
```

## References

- J. Welles Wilder, New Concepts in Technical Trading Systems, Trend Research, 1978
