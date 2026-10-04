# True Range

How far price actually travelled in a bar, counting the gap from the previous
close. It is the widest of three spans: the bar's own high to low, and the
distance from each of them back to where the last bar closed, so an opening gap
is measured rather than ignored.

## Formula

$$
\mathrm{TR}_t = \max\bigl(h_t - l_t,\; |h_t - c_{t-1}|,\; |l_t - c_{t-1}|\bigr)
$$

where $h$, $l$ and $c$ are `high`, `low` and `close`.

## Conventions

- Lookback is 1: the first bar has no previous close, so it has no true range.
- Takes no parameters.
- Leading `NaN` rows are skipped and the lookback counts from the first valid
  bar; a `NaN` or infinity after it raises `InvalidInput` (Deviation 1).
- The value is never negative, because `high >= low` holds on a valid bar.

## Example

```python
import trendlib as tl

span = tl.trange(df)
```

## References

- J. Welles Wilder Jr., *New Concepts in Technical Trading Systems*, Trend
  Research, 1978, chapter 2.
