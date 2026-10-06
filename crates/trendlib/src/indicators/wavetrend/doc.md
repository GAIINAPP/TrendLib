# WaveTrend Oscillator

LazyBear's WaveTrend as finta computes it: how far the typical price sits from
its average, in units of its average distance, smoothed; with a four-bar mean of
that line beside it.

## Formula

With $P_t = (H_t + L_t + C_t)/3$ and $E_\alpha$ the adjusted exponential mean
of everything so far:

$$
e = E_{2/(a+1)}(P) \qquad d = E_{2/(a+1)}(|P - e|) \qquad
\text{ci}_t = \frac{P_t - e_t}{0.015\, d_t}
$$

$$
\text{wavetrend\_1} = E_{2/(b+1)}(\text{ci}) \qquad
\text{wavetrend\_2}_t = \tfrac14 \sum_{j=t-3}^{t} \text{wavetrend\_1}_j
$$

where $a$ is `channel_length` and $b$ is `average_length`.

## Conventions

- Its averages are pandas' `ewm(adjust=True)`, finta's: each is the weighted
  mean of every value so far, the newest weighted 1 and each older one `1 -
  alpha` times the next, so it starts at the first value and never forgets one.
  That makes it path dependent: a slice gives different values from the same
  rows of the whole series.
- The first bar's channel index is 0/0, so both outputs start on row 4, where
  the second line has four values to average; the first line is defined from row
  1 but is held back with it, as every output of an indicator starts on one row.
- Where the typical price has not moved the channel index is 0/0 again and the
  lines keep their last value, as pandas skips a NaN; a series that never moves
  reads NaN.
- finta's parameters are spelled `channel_lenght` and `average_lenght`; these
  are the same parameters.
- Oracle F (`DECISIONS.md` D19): the golden files are `finta` 1.3's `TA.WTO`,
  test-only and never shipped.

## Example

```python
import trendlib as tl
1, 2 = tl.wavetrend(high, low, close)
```

## References

- LazyBear, WaveTrend Oscillator, TradingView, 2014
- finta 1.3, finta.TA (oracle F)
