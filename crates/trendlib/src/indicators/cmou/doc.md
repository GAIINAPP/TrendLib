# Chande Momentum Oscillator (unsmoothed)

Net movement over the window as a percentage of all the movement in it, with plain sums rather than Wilder's smoothing.

## Formula

$$
\mathrm{cmou}_t = 100 \cdot \frac{\sum \text{gains} - \sum \text{losses}}{\sum \text{gains} + \sum \text{losses}}
$$

over the `period` changes ending at $t$.

## Conventions

- Warm-up is `period` bars, one more than the number of changes summed.
- `cmo` is the same figure with Wilder's smoothing in place of the plain sums,
  which makes it recursive where this is not.
- A window that did not move at all gives `0.0`, as TA-Lib does. The test is on
  the exact total.
- Not path dependent.

## Example

```python
import trendlib as tl
cmou = tl.cmou(source)
```

## References

- Tushar S. Chande and Stanley Kroll, The New Technical Trader, Wiley, 1994
