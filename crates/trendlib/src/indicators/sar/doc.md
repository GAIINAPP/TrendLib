# Parabolic SAR

A trailing stop that starts behind price and closes on it a little faster each
time a new extreme is reached. When price reaches the stop, the stop flips to
the other side and starts again.

## Formula

With $\mathrm{EP}$ the best price reached since the last flip and $\alpha$ the
step,

$$
\mathrm{sar}_t = \mathrm{sar}_{t-1} + \alpha\,(\mathrm{EP} - \mathrm{sar}_{t-1})
$$

While rising, the stop is held at or below the lows of the two bars before this
one; while falling, at or above their highs. Each new extreme raises $\alpha$ by
`acceleration` up to `maximum`. When the bar reaches the stop, the direction
flips, $\alpha$ returns to `acceleration` and the new stop is the old extreme,
held clear of the last two bars.

## Conventions

- Warm-up is 1 bar: the first direction is read from the larger of the two
  directional movements over the opening pair, so the first stop lands on the
  second bar.
- `acceleration` is held to `maximum` before anything starts, so a `maximum` of
  zero pins the stop where it began. TA-Lib caps it the same way.
- On the first step only one earlier bar has been seen, so only that one holds
  the stop back; from the second step on, two do.
- Path dependent (`CONVENTIONS.md` § 5): the whole line follows from where the
  first direction was read, so a run started later can be on the other side of
  price.

## Example

```python
import trendlib as tl
stop = tl.sar(high, low, acceleration=0.02, maximum=0.2)
```

## References

- J. Welles Wilder Jr., *New Concepts in Technical Trading Systems*, Trend Research, 1978.
