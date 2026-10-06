# ATR Bands

Bands a multiple of the average true range above and below the close.

## Formula

With $\mathrm{ATR}_n$ TrendLib's `atr`,

$$
\text{upper}_t = C_t + k\,\mathrm{ATR}_n(t) \qquad \text{lower}_t = C_t - k\,\mathrm{ATR}_n(t)
$$

where $n$ is `period` and $k$ is `shift`.

## Conventions

- Warm-up is `period` bars, the average true range's.
- The bands are centred on the close, ChartIQ's default field.
- Recursive through the average true range's Wilder smoothing.
- Oracle A (`DECISIONS.md` D20): the golden files evaluate this formula through
  TA-Lib's `ATR`, `MULT`, `ADD` and `SUB`.

## Example

```python
import trendlib as tl
bands_upper, bands_lower = tl.atr_bands(high, low, close)
```

## References

- ChartIQ, ATR Bands
