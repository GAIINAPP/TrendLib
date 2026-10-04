# Vortex Indicator

How much of the recent true range was covered reaching up past the last low and down past the last high.

## Formula

$$
\mathrm{+VI}_t = \frac{\sum |H_i - L_{i-1}|}{\sum \mathrm{TR}_i} \qquad
\mathrm{-VI}_t = \frac{\sum |L_i - H_{i-1}|}{\sum \mathrm{TR}_i}
$$

over the `period` bars ending at $t$.

## Conventions

- Warm-up is `period` bars: each movement reaches back to the previous bar, so
  the window needs one bar more than it has movements.
- A window whose true ranges sum to zero reads `0.0` on both lines, as TA-Lib
  does. The test is exact.
- Both sums are re-added from the window each bar rather than carried forward.
- Not path dependent.

## Example

```python
import trendlib as tl
vortex_plusvi, vortex_minusvi = tl.vortex(high, low, close)
```

## References

- Etienne Botes and Douglas Siepman, The Vortex Indicator, Technical Analysis of Stocks and Commodities, January 2010
