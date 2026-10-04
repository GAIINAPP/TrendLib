# Parabolic SAR Extended

The parabolic stop with each side given its own step schedule, a starting point
that can be chosen, and a push away from price when it flips. The sign of the
output says which side the stop is on.

## Formula

The same recursion as `sar`,

$$
\mathrm{sarext}_t = \mathrm{sarext}_{t-1} + \alpha\,(\mathrm{EP} - \mathrm{sarext}_{t-1})
$$

with a separate $\alpha$ schedule for the rising and the falling side. On a flip
the stop becomes the old extreme, held clear of the last two bars, and is then
pushed a further `offset_on_reverse` of itself away from price.

## Conventions

- Warm-up is 1 bar.
- The value is **negative while the stop sits above price**, which is how the
  direction is reported. `sar` returns the same magnitudes unsigned.
- `start_value` of zero reads the first direction from the opening pair, as
  `sar` does. A positive value starts the stop there and rising, a negative one
  starts it at its magnitude and falling.
- The offset moves the stop itself, not only what is reported, so the next bar
  advances from the pushed value.
- Each `acceleration_init` and `acceleration` is held to its own maximum before
  anything starts, so a maximum of zero pins that side's stop where it began.
- On the first bar the stop has only that bar to be held back by; from the
  second on, two bars hold it.
- Path dependent (`CONVENTIONS.md` § 5): the whole line follows from where the
  first direction was read.

## Example

```python
import trendlib as tl
stop = tl.sarext(high, low, acceleration_init_long=0.02, acceleration_max_long=0.2)
```

## References

- J. Welles Wilder Jr., *New Concepts in Technical Trading Systems*, Trend Research, 1978.
