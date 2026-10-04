# Detrended Price Oscillator

How far the series sat above or below its own moving average half a period
ago. Comparing against the average that is centred on that earlier bar takes
the trend out, leaving the shorter swings around it.

## Formula

$$
\mathrm{dpo}_t = x_{t - s} - \mathrm{SMA}(x, n)_t
\qquad s = \left\lfloor \tfrac{n}{2} \right\rfloor + 1
$$

where $x$ is `source` and $n$ is `period`.

## Conventions

- Warm-up is the larger of `period - 1` and the shift, which is the shift at
  periods below 4 and the average's warm-up above them.
- The value is not placed at the bar it describes: it sits at row `t` while the
  value it is built from is `s` bars earlier. Shifting the series to centre it
  is a charting choice, not part of the number.
- Not path dependent.

## Example

```python
import trendlib as tl
detrended = tl.dpo(close, period=20)
```

## References

- Steven B. Achelis, *Technical Analysis from A to Z*, McGraw-Hill, 2000.
