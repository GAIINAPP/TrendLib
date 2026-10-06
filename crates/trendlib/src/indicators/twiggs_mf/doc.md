# Twiggs Money Flow

Colin Twiggs' money flow: Chaikin's close-location volume measured against the
true range rather than the bar's range, Wilder-smoothed and divided by
Wilder-smoothed volume. It reads from -1 to 1.

## Formula

From bar 1, with $\overline H_t = \max(H_t, C_{t-1})$ and
$\underline L_t = \min(L_t, C_{t-1})$,

$$
A_t = \frac{(C_t - \underline L_t) - (\overline H_t - C_t)}{\overline H_t - \underline L_t}\,V_t
\qquad
\text{twiggs\_mf}_t = \frac{W_n(A)_t}{W_n(V)_t}
$$

where $W_n$ is Wilder's smoothing over $n$ = `period` bars, seeded with the
simple mean of bars 1 to $n$, and $V$ is the volume.

## Conventions

- A bar whose true range is zero contributes $A_t = 0$; where the smoothed
  volume is zero the output is 0 rather than 0/0.
- Both averages start at bar 1, the first with a previous close, so warm-up is
  `period` bars.
- Twiggs smooths with Wilder's average (a 21-bar Wilder average is a 41-bar
  exponential one), as `rma` does.
- Volume must not be negative.
- Recursive through the two Wilder averages.
- Oracle A (`DECISIONS.md` D20): the golden files evaluate this formula through
  TA-Lib's `SMA` (the seeds), with the true-range bounds, Wilder's step and the
  ratio in NumPy.

## Example

```python
import trendlib as tl
twiggs_mf = tl.twiggs_mf(high, low, close, volume)
```

## References

- Colin Twiggs, Twiggs Money Flow, Incredible Charts
