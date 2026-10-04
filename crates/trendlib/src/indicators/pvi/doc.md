# Positive Volume Index

An index that only moves on bars where volume rises, so it follows what happened on the quieter or busier days alone.

## Formula

$$
\mathrm{pvi}_t =
\begin{cases}
\mathrm{pvi}_{t-1}\left(1 + \dfrac{C_t - C_{t-1}}{C_{t-1}}\right) & V_t > V_{t-1}\\[2mm]
\mathrm{pvi}_{t-1} & \text{otherwise}
\end{cases}
$$

## Conventions

- Lookback 0: the first bar starts the index at 1000.
- Path dependent (`CONVENTIONS.md` § 5): the value carries every bar before it,
  so a run started later begins its own total from zero.
- Equal volume on two bars moves nothing: the comparison is strict.
- `nvi` is the same construction reading the other kind of bar.

## Example

```python
import trendlib as tl
pvi = tl.pvi(close, volume)
```

## References

- Paul L. Dysart, as described in Norman G. Fosback, Stock Market Logic, Dearborn, 1976
