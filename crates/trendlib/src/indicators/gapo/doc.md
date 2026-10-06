# Gopalakrishnan Range Index

Jayanthi Gopalakrishnan's range index: the logarithm of the highest high less
the lowest low over a window, divided by the logarithm of the window's length.

## Formula

$$
\text{gapo}_t = \frac{\ln\big(\max_{t-n < j \le t} H_j - \min_{t-n < j \le t} L_j\big)}{\ln n}
$$

where $n$ is `period`.

## Conventions

- A window whose high and low never part has a range of 0 and reads negative
  infinity, the logarithm of 0. A range below 1 price unit reads below 0.
- Warm-up is `period - 1` bars.
- Not path dependent.
- Oracle A (`DECISIONS.md` D20): the golden files evaluate this formula through
  TA-Lib's `MAX`, `MIN`, `SUB`, `LN` and `DIV`.

## Example

```python
import trendlib as tl
gapo = tl.gapo(high, low)
```

## References

- Jayanthi Gopalakrishnan, Technical Analysis of Stocks & Commodities, January 2001
