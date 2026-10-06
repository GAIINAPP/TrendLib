# Woodies CCI

Ken Wood's pair of Commodity Channel Indexes over the typical price: a 14-bar
main line and a 6-bar turbo line drawn together.

## Formula

With $\mathrm{CCI}_n$ TrendLib's `cci` (Lambert's index over the typical price,
scaled by 0.015 of the mean deviation),

$$
\text{woodies\_cci}_t = \mathrm{CCI}_{a}(t) \qquad \text{woodies\_cci\_turbo}_t = \mathrm{CCI}_{b}(t)
$$

where $a$ is `cci_period` and $b$ is `turbo_period`.

## Conventions

- Both lines start on the same row, the longer window's warm-up:
  `max(cci_period, turbo_period) - 1`, 13 at the defaults.
- A window whose typical price never moved reads 0, as `cci` does.
- Not path dependent.
- Oracle A (`DECISIONS.md` D20): the golden files evaluate this formula through
  TA-Lib's `CCI`.

## Example

```python
import trendlib as tl
cci, cci_turbo = tl.woodies_cci(high, low, close)
```

## References

- Ken Wood, Woodies CCI Club
- Donald Lambert, Commodity Channel Index, Commodities, 1980
