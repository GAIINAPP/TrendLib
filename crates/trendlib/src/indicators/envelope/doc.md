# Moving Average Envelope

A simple moving average with bands a fixed percentage above and below it.

## Formula

$$
\text{middle}_t = \frac1n \sum_{j=0}^{n-1} x_{t-j} \qquad
\text{upper}_t = \text{middle}_t \Big(1 + \frac{p}{100}\Big) \qquad
\text{lower}_t = \text{middle}_t \Big(1 - \frac{p}{100}\Big)
$$

where $x$ is `source`, $n$ is `period` and $p$ is `percent`.

## Conventions

- Warm-up is `period - 1` bars, the simple average's.
- The average is simple, TradingView's default for its Envelope; `percent` is a
  percentage, so 10 puts each band a tenth of the average away.
- Not path dependent.
- Oracle A (`DECISIONS.md` D20): the golden files evaluate this formula through
  TA-Lib's `SMA` and `MULT`.

## Example

```python
import trendlib as tl
upper, middle, lower = tl.envelope(source)
```

## References

- TradingView, Envelope
