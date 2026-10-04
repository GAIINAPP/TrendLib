# Donchian Channel

The highest high and the lowest low of the last `period` bars, with the
midpoint between them. The bands only move when a bar reaches past the ones
before it, so they sit still through a range and step when it breaks.

## Formula

$$
\text{upper}_t = \max_{i \in [t-n+1,\,t]} H_i \qquad
\text{lower}_t = \min_{i \in [t-n+1,\,t]} L_i \qquad
\text{middle}_t = \frac{\text{upper}_t + \text{lower}_t}{2}
$$

where $n$ is `period`.

## Conventions

- Warm-up is `period - 1` bars.
- The window includes the current bar, so the upper band is never below the
  current high. Implementations that exclude the current bar give a different
  channel; this one follows TA-Lib.
- Not path dependent.

## Example

```python
import trendlib as tl
upper, middle, lower = tl.donchian(high, low, period=20)
```

## References

- Richard Donchian, "Trend Following Methods in Commodity Price Analysis", *Commodity Year Book*, 1957.
