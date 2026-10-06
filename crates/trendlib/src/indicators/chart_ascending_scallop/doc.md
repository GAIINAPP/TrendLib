# Ascending Scallop

Marks a close above a J-shaped dip and recovery of the last `period` closes
while price stands above where the window began: the rising-trend scallop of the
oracle.

## Formula

Over the closes of the `period` bars before $t$, split into a first
third (`period / 3` closes), a middle third (the next `period / 3`) and the rest,
let $A$, $M$ and $Z$ be their means.

$$
\text{chart\_ascending\_scallop}_t = +100\,\big[M < A \wedge Z > M \wedge Z \ge 0.95\,A \wedge C_t > C_{t-p} \wedge C_t > \max_{t-p \le j < t} C_j\big]
$$

where $p$ is `period`.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `scallop_asc` with
  `mode="confirmed"` (oracle P, `INDICATORS.md` § 5.2); `period` is its
  `window`.
- The 5 percent band on the last third is the oracle's constant, not a
  parameter.
- The reading is repeated on every bar the conditions hold, not only the first.
- Warm-up is `period` bars, the first bar the oracle reads. An `int32` column
  warms up with `0`, not `NaN` (`CONVENTIONS.md` § 2), and the output says what
  price did relative to the shape; it names no price level.
- Not path dependent: the answer depends on the bars in the window and nothing
  before them.

## Example

```python
import trendlib as tl
found = tl.chart_ascending_scallop(high, low, close)
```

## References

- Thomas N. Bulkowski, Encyclopedia of Chart Patterns, 2nd edition, Wiley, 2005
- ta-patterns 1.2.1, ta_patterns.chart_patterns.scallop_asc (oracle P)
