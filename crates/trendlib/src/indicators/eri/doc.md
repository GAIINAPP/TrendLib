# Elder Ray Index

How far each bar's high and low reached past an exponential average of the closes.

## Formula

$$
\text{bull}_t = H_t - \mathrm{EMA}(C, n)_t \qquad
\text{bear}_t = L_t - \mathrm{EMA}(C, n)_t
$$

## Conventions

- Warm-up is `period - 1` bars.
- Both readings use the same average, so their difference is always the bar's
  own range.
- Recursive, so the result is path dependent (`CONVENTIONS.md` § 5);
  `unstable` applies.

## Example

```python
import trendlib as tl
eri_bull_power, eri_bear_power = tl.eri(high, low, close)
```

## References

- Alexander Elder, Trading for a Living, Wiley, 1993
