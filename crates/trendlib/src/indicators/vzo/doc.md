# Volume Zone Oscillator

Walid Khalil's Volume Zone Oscillator as finta computes it: the average of
volume signed by whether the close rose or fell, as a percentage of the average
volume.

## Formula

With $E_\alpha$ the adjusted exponential mean of everything so far and $V$ the
volume:

$$
\text{vzo}_t = 100\, \frac{E_{2/(p+1)}\big(\operatorname{sign}(C - C_{-1})\, V\big)_t}{E_{2/(p+1)}(V)_t}
$$

where $p$ is `period` and the first bar's sign is 0.

## Conventions

- Its averages are pandas' `ewm(adjust=True)`, finta's: each is the weighted
  mean of every value so far, the newest weighted 1 and each older one `1 -
  alpha` times the next, so it starts at the first value and never forgets one.
  That makes it path dependent: a slice gives different values from the same
  rows of the whole series.
- The first bar has no previous close; its signed volume is 0, finta's sign of a
  missing change.
- Volume must not be negative. Where no volume has traded yet the ratio is 0/0
  and the output NaN.
- Oracle F (`DECISIONS.md` D19): the golden files are `finta` 1.3's `TA.VZO`,
  test-only and never shipped.

## Example

```python
import trendlib as tl
vzo = tl.vzo(close, volume)
```

## References

- Walid Khalil and David Steckler, In the Volume Zone, Technical Analysis of Stocks & Commodities, May 2011
- finta 1.3, finta.TA (oracle F)
