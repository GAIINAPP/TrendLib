# Volume Weighted Average Price

The average price traded so far in the session, each bar weighted by the volume
that went through it. It starts over when the session does, so it answers "what
has the average fill been today" rather than "over the last n bars".

## Formula

With $\mathrm{TP}_t = (H_t + L_t + C_t)/3$,

$$
\mathrm{vwap}_t =
\frac{\sum_{i \in \text{session}(t),\ i \le t} \mathrm{TP}_i\,V_i}
     {\sum_{i \in \text{session}(t),\ i \le t} V_i}
$$

With `anchor="day"` a session is the bars sharing one calendar date; with
`anchor="none"` the whole series is one session.

## Conventions

- Lookback 0: the first bar of a session is its own average.
- Before any volume has traded in a session the row is `NaN`, not `0`: a zero
  is not a price (`CONVENTIONS.md` deviation 2). A zero-volume bar after volume
  has traded leaves the value unchanged.
- TA-Lib's VWAP never resets and expects the caller to slice sessions itself;
  the session anchor is the deviation this library ships
  (`CONVENTIONS.md` deviation 3). `tl.VWAP`, the uppercase alias, is TA-Lib's
  behaviour: `anchor="none"`.
- The session boundary is the calendar date of the timestamp as given. A
  caller wanting another zone passes timestamps already expressed in it; a `tz`
  parameter is M6 (`DECISIONS.md`).
- Path dependent (`CONVENTIONS.md` § 5): the value carries the session so far,
  so a run started mid-session begins its own total there.

## Example

```python
import trendlib as tl
session = tl.vwap(high, low, close, volume, timestamps)
```

## References

- `INDICATORS.md` section 3.1, the approved session-anchored definition.
