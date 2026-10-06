# High and Tight Flag

Marks a breakout from a high and tight flag: closes rose at least `min_pole`
over `pole_bars` bars, the `period` bars since stayed within `max_retrace` of
the pole's last close, and the close is now above every high of those bars.

## Formula

With $e = t - p$ the pole's last bar, $s = e - q$ its first,
$F^H = \max_{e \le j < t} H_j$ and $F^L = \min_{e \le j < t} L_j$:

$$
\text{chart\_high\_tight\_flag}_t = 100\,\Big[\frac{C_e - C_s}{C_s} \ge \mu
\wedge \frac{F^H - F^L}{C_e} \le \rho \wedge C_t > F^H\Big]
$$

where $p$ is `period`, $q$ is `pole_bars`, $\mu$ is `min_pole` and $\rho$ is
`max_retrace`.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `flag_high_tight`
  with `mode="confirmed"` (oracle P, `INDICATORS.md` § 5.2); `period` is its
  `window`.
- Unlike `chart_bull_flag` it draws no trendlines: the flag is the range of the
  bars since the pole, as in the oracle.
- The reading is repeated on every bar the conditions hold, not only the first.
- Warm-up is `period + pole_bars` bars, the first bar the oracle reads. An
  `int32` column warms up with `0`, not `NaN` (`CONVENTIONS.md` § 2), and the
  output says what price did relative to the shape; it names no price level.
- Not path dependent: the answer depends on the bars in the window and nothing
  before them.

## Example

```python
import trendlib as tl
found = tl.chart_high_tight_flag(high, low, close)
```

## References

- Thomas N. Bulkowski, Encyclopedia of Chart Patterns, 2nd edition, Wiley, 2005
- ta-patterns 1.2.1, ta_patterns.chart_patterns.flag_high_tight (oracle P)
