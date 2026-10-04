# Forecast Oscillator

How far the value came in above or below what the previous bar's fitted line forecast for it.

## Formula

$$
\mathrm{fosc}_t = 100 \cdot \frac{x_t - \mathrm{tsf}_{t-1}}{x_t}
$$

where $\mathrm{tsf}$ is the least squares line fitted over `period` bars and
carried one bar forward.

## Conventions

- Warm-up is `period` bars: `period - 1` for the fit and one more to hold it
  back to the bar it was forecasting.
- A value of exactly zero makes the percentage non-finite rather than guarded.
- Not path dependent.

## Example

```python
import trendlib as tl
fosc = tl.fosc(source)
```

## References

- Tushar S. Chande and Stanley Kroll, The New Technical Trader, Wiley, 1994
