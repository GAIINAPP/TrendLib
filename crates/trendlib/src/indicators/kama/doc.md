# Kaufman Adaptive Moving Average

A moving average that changes how hard it smooths according to how directly
the series has been moving. A run that travels a long way to end up where it
started is smoothed heavily; one that goes straight there is barely smoothed at
all.

## Formula

$$
\mathrm{ER}_t = \frac{|x_t - x_{t-n}|}{\sum_{i=t-n+1}^{t} |x_i - x_{i-1}|}
\qquad
\alpha_t = \big(\mathrm{ER}_t\,(f - s) + s\big)^2
$$

$$
\mathrm{kama}_t = \mathrm{kama}_{t-1} + \alpha_t\,(x_t - \mathrm{kama}_{t-1})
$$

where $n$ is `period`, $f = 2/3$ and $s = 2/31$.

## Conventions

- The two ends of the smoothing range are fixed at the constants for periods 2
  and 30 whatever `period` is. `period` only sets how far back efficiency is
  measured.
- The average is seeded with the bar before its first value, not with an
  average of the window, so warm-up is `period` bars.
- The ratio cannot exceed 1: a window cannot end further from where it began
  than the distance it covered. Rounding can put it a hair over, and a window
  that did not move at all puts it at `0/0`; both are read as fully efficient,
  which is the fastest smoothing. A flat stretch therefore keeps the average
  pinned to the series rather than letting it drift.
- With `period=1` the ratio would always be 1 and the average would smooth at
  a fixed rate rather than adapt, so the series is returned unchanged with a
  lookback of zero. That is what TA-Lib does.
- Recursive, so the result is path dependent (`CONVENTIONS.md` § 5);
  `unstable` applies.

## Example

```python
import trendlib as tl
adaptive = tl.kama(close, period=30)
```

## References

- Perry J. Kaufman, *Smarter Trading*, McGraw-Hill, 1995.
