# Elder SafeZone

Alexander Elder's SafeZone levels: the prior bar's low less a multiple of the
average amount recent lows have undercut the low before them, and the prior high
plus the same for highs, each held from moving against the trend for a few bars.

## Formula

From bar 1, a downside penetration is $d_u = L_{u-1} - L_u$ when $L_u < L_{u-1}$
and 0 otherwise, and an upside one $e_u = H_u - H_{u-1}$ when $H_u > H_{u-1}$. With
$\bar d_u$ the mean of the nonzero $d$ among bars $u-n+1 \dots u$ (0 if there are
none), and $\bar e_u$ the same for $e$,

$$
\ell_t = L_{t-1} - k\,\bar d_{t-1} \qquad
\text{lower}_t = \max_{t-h < j \le t} \ell_j
$$

$$
\upsilon_t = H_{t-1} + k\,\bar e_{t-1} \qquad
\text{upper}_t = \min_{t-h < j \le t} \upsilon_j
$$

where $n$ is `period`, $k$ is `coefficient` and $h$ is `hold`.

## Conventions

- Row $t$ holds the levels for bar $t$, built from the bars before it: the prior
  bar's low or high and the penetrations up to it, as Elder computes tomorrow's
  level tonight.
- The average counts only the bars that did penetrate; a window with none
  averages 0, which puts the level at the prior low or high itself.
- `hold` is Elder's rule that the lower level never falls and the upper never
  rises over that many bars, taken as the highest (lowest) of the last `hold`
  raw levels.
- Elder calls these stops; TrendLib reports them as levels (D11).
- Warm-up is `period + hold` bars, 13 at the defaults.
- Not path dependent: every value depends on a fixed window of bars.
- Oracle A (`DECISIONS.md` D20): the golden files evaluate this formula through
  TA-Lib's `SUM`, `MAX` and `MIN`, the penetrations and their mean in NumPy.

## Example

```python
import trendlib as tl
lower, upper = tl.safezone(high, low)
```

## References

- Alexander Elder, Come Into My Trading Room, Wiley, 2002
