use crate::core::chart::whole;
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::{Lagged, RollingWindow};
use crate::indicators::rsi;

pub const NAME: &str = "rsi_divergence";

pub const PERIOD_DEFAULT: usize = 14;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;
pub const LEFT_BARS_DEFAULT: usize = 5;
pub const LEFT_BARS_MIN: usize = 1;
pub const LEFT_BARS_MAX: usize = 100_000;
pub const RIGHT_BARS_DEFAULT: usize = 5;
pub const RIGHT_BARS_MIN: usize = 1;
pub const RIGHT_BARS_MAX: usize = 100_000;
pub const MIN_BARS_BETWEEN_DEFAULT: usize = 5;
pub const MIN_BARS_BETWEEN_MIN: usize = 0;
pub const MIN_BARS_BETWEEN_MAX: usize = 100_000;
pub const MAX_BARS_BETWEEN_DEFAULT: usize = 60;
pub const MAX_BARS_BETWEEN_MIN: usize = 0;
pub const MAX_BARS_BETWEEN_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub period: usize,
    pub left_bars: usize,
    pub right_bars: usize,
    pub min_bars_between: usize,
    pub max_bars_between: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
            left_bars: LEFT_BARS_DEFAULT,
            right_bars: RIGHT_BARS_DEFAULT,
            min_bars_between: MIN_BARS_BETWEEN_DEFAULT,
            max_bars_between: MAX_BARS_BETWEEN_DEFAULT,
        }
    }
}

/// How far apart two RSI readings must be to count as different. Readings that
/// are equal in exact arithmetic come out a unit or two in the last place apart
/// (an unchanged close leaves RSI where it was, a run of rises holds it at 100),
/// and which way the rounding falls must not decide whether a bar is a pivot or
/// a divergence. RSI runs from 0 to 100, so this is far below any real move.
const TIE: f64 = 1e-9;

/// A turning point of the RSI, and the price on its bar: the low for a trough,
/// the high for a peak.
#[derive(Clone, Copy, Debug)]
struct Pivot {
    at: usize,
    rsi: f64,
    price: f64,
}

/// What one confirmation bar found: the two codes, and the pivot it
/// confirmed, if any, with whether it is a peak.
type Found = ([f64; 2], Option<(bool, Pivot)>);

#[derive(Clone, Debug)]
pub struct State {
    rsi: rsi::State,
    /// The last `left_bars + right_bars + 1` RSI readings; the candidate pivot
    /// is the one `right_bars` from the newest.
    readings: RollingWindow,
    /// The high and the low `right_bars` bars back, on the candidate's bar.
    highs: Lagged,
    lows: Lagged,
    left: usize,
    right: usize,
    min_between: usize,
    max_between: usize,
    last_peak: Option<Pivot>,
    last_trough: Option<Pivot>,
    bars: usize,
}

impl State {
    /// The codes a newly confirmed pivot gives against the previous pivot of
    /// its side. Only the previous one is compared, and only when the bars
    /// strictly between the two are within the range.
    fn codes(&self, previous: Option<Pivot>, now: Pivot, peak: bool) -> [f64; 2] {
        let Some(before) = previous else {
            return [0.0; 2];
        };
        let between = now.at - before.at - 1;
        if between < self.min_between || between > self.max_between {
            return [0.0; 2];
        }
        let (price_up, price_down) = (now.price > before.price, now.price < before.price);
        let (rsi_up, rsi_down) = (now.rsi > before.rsi + TIE, now.rsi < before.rsi - TIE);
        let code = |hit: bool, sign: f64| if hit { sign * 100.0 } else { 0.0 };
        if peak {
            [
                code(price_up && rsi_down, -1.0),
                code(price_down && rsi_up, -1.0),
            ]
        } else {
            [
                code(price_down && rsi_up, 1.0),
                code(price_up && rsi_down, 1.0),
            ]
        }
    }

    /// Reads a full window of RSI readings whose candidate sits on bar `at`.
    /// The comparison is strict, so a reading that ties a neighbour (within
    /// `TIE`) is not a pivot and a flat RSI marks nothing.
    fn read<I: Iterator<Item = f64>>(
        &self,
        window: impl Fn() -> I,
        at: usize,
        high: f64,
        low: f64,
    ) -> Found {
        let Some(middle) = window().nth(self.left) else {
            return ([0.0; 2], None);
        };
        let (mut peak, mut trough) = (true, true);
        for (offset, value) in window().enumerate() {
            if offset != self.left {
                peak &= middle > value + TIE;
                trough &= middle < value - TIE;
            }
        }
        if peak {
            let pivot = Pivot {
                at,
                rsi: middle,
                price: high,
            };
            (self.codes(self.last_peak, pivot, true), Some((true, pivot)))
        } else if trough {
            let pivot = Pivot {
                at,
                rsi: middle,
                price: low,
            };
            (
                self.codes(self.last_trough, pivot, false),
                Some((false, pivot)),
            )
        } else {
            ([0.0; 2], None)
        }
    }
}

impl Step<3, 2> for State {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 2]> {
        let now = self.bars;
        self.bars += 1;
        let high = self.highs.push(bar[0]);
        let low = self.lows.push(bar[1]);
        let [reading] = self.rsi.push([bar[2]])?;
        if !self.readings.push(reading) {
            return None;
        }
        let (out, found) = self.read(|| self.readings.values(), now - self.right, high?, low?);
        match found {
            Some((true, pivot)) => self.last_peak = Some(pivot),
            Some((false, pivot)) => self.last_trough = Some(pivot),
            None => {}
        }
        Some(out)
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 2]> {
        let [reading] = self.rsi.preview([bar[2]])?;
        if !self.readings.preview_is_full() {
            return None;
        }
        let (out, _) = self.read(
            || self.readings.preview_values(reading),
            self.bars - self.right,
            self.highs.earlier()?,
            self.lows.earlier()?,
        );
        Some(out)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RsiDivergence;

pub type RsiDivergenceStream = BarStream<RsiDivergence, 3, 2>;

impl Kernel<3, 2> for RsiDivergence {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 2] = ["rsi_divergence", "rsi_hidden_divergence"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(NAME, "period", params.period, PERIOD_MIN, PERIOD_MAX)?;
        whole(
            NAME,
            "left_bars",
            params.left_bars,
            LEFT_BARS_MIN,
            LEFT_BARS_MAX,
        )?;
        whole(
            NAME,
            "right_bars",
            params.right_bars,
            RIGHT_BARS_MIN,
            RIGHT_BARS_MAX,
        )?;
        whole(
            NAME,
            "min_bars_between",
            params.min_bars_between,
            MIN_BARS_BETWEEN_MIN,
            MIN_BARS_BETWEEN_MAX,
        )?;
        whole(
            NAME,
            "max_bars_between",
            params.max_bars_between,
            MAX_BARS_BETWEEN_MIN,
            MAX_BARS_BETWEEN_MAX,
        )
    }

    /// The RSI's own warm-up, then the readings either side of the first bar
    /// that can be a pivot.
    fn lookback(params: &Params) -> usize {
        params.period + params.left_bars + params.right_bars
    }

    fn state(params: &Params) -> State {
        State {
            rsi: rsi::Rsi::state(&rsi::Params {
                period: params.period,
            }),
            readings: RollingWindow::new(params.left_bars + params.right_bars + 1),
            highs: Lagged::new(params.right_bars),
            lows: Lagged::new(params.right_bars),
            left: params.left_bars,
            right: params.right_bars,
            min_between: params.min_bars_between,
            max_between: params.max_bars_between,
            last_peak: None,
            last_trough: None,
            bars: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Params, RsiDivergence, RsiDivergenceStream};
    use crate::core::kernel::Kernel;
    use crate::core::traits::Stream;

    /// Closes from 100 along `legs` of (bars, step), with a high and a low half
    /// a point either side.
    fn bars(legs: &[(usize, f64)]) -> [Vec<f64>; 3] {
        let mut close = vec![100.0];
        for &(count, step) in legs {
            for _ in 0..count {
                close.push(close.last().unwrap() + step);
            }
        }
        let high = close.iter().map(|c| c + 0.5).collect();
        let low = close.iter().map(|c| c - 0.5).collect();
        [high, low, close]
    }

    fn params() -> Params {
        Params {
            period: 14,
            left_bars: 2,
            right_bars: 2,
            min_bars_between: 0,
            max_bars_between: 60,
        }
    }

    fn run(bars: &[Vec<f64>; 3], params: &Params) -> [Vec<f64>; 2] {
        RsiDivergence::batch([&bars[0], &bars[1], &bars[2]], params).unwrap()
    }

    /// The bars a reading fires on, with its code.
    fn fired(column: &[f64]) -> Vec<(usize, f64)> {
        column
            .iter()
            .enumerate()
            .filter(|(_, v)| **v != 0.0 && !v.is_nan())
            .map(|(at, v)| (at, *v))
            .collect()
    }

    // A steep fall to 102 (bar 26), a rally, then a shorter fall to a lower
    // low of 101 (bar 33) on which RSI holds higher: confirmed two bars later.
    const REGULAR: [(usize, f64); 5] = [(20, 1.0), (6, -3.0), (4, 2.0), (3, -3.0), (8, 2.0)];

    // A dip to 114 (bar 23), a rally, then a dip to a higher low of 115
    // (bar 30) on which RSI sinks lower.
    const HIDDEN: [(usize, f64); 5] = [(20, 1.0), (3, -2.0), (4, 1.0), (3, -1.0), (8, 2.0)];

    #[test]
    fn a_lower_low_on_a_higher_rsi_low_is_a_regular_bullish_divergence() {
        let [regular, hidden] = run(&bars(&REGULAR), &params());
        assert_eq!(fired(&regular), [(35, 100.0)]);
        assert_eq!(fired(&hidden), []);
    }

    #[test]
    fn a_higher_low_on_a_lower_rsi_low_is_a_hidden_bullish_divergence() {
        let [regular, hidden] = run(&bars(&HIDDEN), &params());
        assert_eq!(fired(&regular), []);
        assert_eq!(fired(&hidden), [(32, 100.0)]);
    }

    #[test]
    fn mirrored_prices_read_bearish_on_the_same_bars() {
        // Reflecting prices turns RSI r into 100 - r and troughs into peaks, so
        // every bullish reading becomes the bearish one, bar for bar.
        for legs in [REGULAR, HIDDEN] {
            let [high, low, close] = bars(&legs);
            let flip = |v: &Vec<f64>| v.iter().map(|x| 400.0 - x).collect::<Vec<f64>>();
            let mirrored = [flip(&low), flip(&high), flip(&close)];
            let [regular, hidden] = run(&[high, low, close], &params());
            let [m_regular, m_hidden] = run(&mirrored, &params());
            for (a, b) in regular
                .iter()
                .zip(&m_regular)
                .chain(hidden.iter().zip(&m_hidden))
            {
                assert!(a.is_nan() && b.is_nan() || *a == -*b, "{a} vs {b}");
            }
        }
    }

    #[test]
    fn pivots_too_far_apart_are_not_compared() {
        // Six bars lie strictly between the two troughs of REGULAR.
        let data = bars(&REGULAR);
        for (max_between, expected) in [(6, vec![(35, 100.0)]), (5, vec![])] {
            let narrow = Params {
                max_bars_between: max_between,
                ..params()
            };
            assert_eq!(
                fired(&run(&data, &narrow)[0]),
                expected,
                "max {max_between}"
            );
        }
        let wide = Params {
            min_bars_between: 7,
            ..params()
        };
        assert_eq!(fired(&run(&data, &wide)[0]), []);
    }

    #[test]
    fn warm_up_is_nan_and_then_every_row_reads() {
        let data = bars(&REGULAR);
        let lookback = RsiDivergence::lookback(&params());
        assert_eq!(lookback, 18);
        for column in run(&data, &params()) {
            assert!(column[..lookback].iter().all(|v| v.is_nan()));
            assert!(
                column[lookback..]
                    .iter()
                    .all(|v| [-100.0, 0.0, 100.0].contains(v))
            );
        }
    }

    #[test]
    fn a_flat_series_has_no_pivots() {
        let flat = vec![250.0; 120];
        for column in run(&[flat.clone(), flat.clone(), flat], &params()) {
            assert!(column.iter().all(|v| v.is_nan() || *v == 0.0));
        }
    }

    #[test]
    fn the_stream_matches_the_batch_and_peek_predicts_update() {
        // A seeded random walk read with short settings, so divergences of all
        // four kinds turn up often and a peek that read anything differently
        // from an update would show.
        let mut state: u64 = 1;
        let mut close = vec![200.0];
        for _ in 1..600 {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let step = ((state >> 33) as f64 / 2_147_483_648.0 - 0.5) * 4.0;
            close.push(close.last().unwrap() + step);
        }
        let high: Vec<f64> = (0..600)
            .map(|i| close[i] + 0.4 + (i % 3) as f64 * 0.3)
            .collect();
        let low: Vec<f64> = (0..600)
            .map(|i| close[i] - 0.4 - (i % 5) as f64 * 0.2)
            .collect();
        let short = Params {
            period: 5,
            left_bars: 1,
            right_bars: 1,
            min_bars_between: 0,
            max_bars_between: 30,
        };
        let batch = run(&[high.clone(), low.clone(), close.clone()], &short);
        for column in &batch {
            for code in [-100.0, 100.0] {
                assert!(column.contains(&code), "the path never read {code}");
            }
        }
        let start = RsiDivergence::lookback(&short) + 1;
        let history = [&high[..start], &low[..start], &close[..start]];
        let mut stream = RsiDivergenceStream::open(history, &short).unwrap();
        for row in start..close.len() {
            let bar = [high[row], low[row], close[row]];
            let peeked = stream.peek(bar).unwrap();
            let updated = stream.update(bar).unwrap();
            assert_eq!(peeked, updated, "bar {row}");
            assert_eq!(updated, [batch[0][row], batch[1][row]], "bar {row}");
        }
    }
}
