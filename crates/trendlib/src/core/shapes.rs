//! The chart shapes of `docs/INDICATORS.md` section 5.2 that are not drawn
//! with two trendlines: shapes read off a window of closes or bars, the
//! diamond's two fits, the bump-and-run's lead-in line and the measured move's
//! two legs. Each is ta-patterns' rule (oracle P), written so a stream and a
//! batch run share it.

use crate::core::bars::BarHistory;
use crate::core::candles::Candle;
use crate::core::chart::{Line, Swing, SwingFinder, SwingLog, Swings};
use crate::core::kernel::Step;

/// A chart shape asked of the last bars, read from high, low and close.
pub type HlcRule<P> = fn(&BarHistory, &P) -> f64;

/// A rule over the last `depth` bars of high, low and close, answering from
/// bar `lookback` on. The bar pattern engine's three-input twin.
#[derive(Clone, Debug)]
pub struct HlcState<P: Copy> {
    history: BarHistory,
    params: P,
    lookback: usize,
    rule: HlcRule<P>,
}

impl<P: Copy> HlcState<P> {
    pub fn new(depth: usize, lookback: usize, params: P, rule: HlcRule<P>) -> Self {
        Self {
            history: BarHistory::new(depth.max(lookback)),
            params,
            lookback,
            rule,
        }
    }
}

impl<P: Copy + std::fmt::Debug> Step<3, 1> for HlcState<P> {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 1]> {
        self.history.push(Candle {
            open: f64::NAN,
            high: bar[0],
            low: bar[1],
            close: bar[2],
        });
        (self.history.seen() > self.lookback).then(|| [(self.rule)(&self.history, &self.params)])
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 1]> {
        self.clone().push(bar)
    }
}

/// The `period` closes before the current bar, oldest first.
#[derive(Clone, Debug)]
pub struct Closes {
    values: Box<[f64]>,
    head: usize,
}

impl Closes {
    fn new(period: usize) -> Self {
        Self {
            values: vec![0.0; period].into_boxed_slice(),
            head: 0,
        }
    }

    fn push(&mut self, close: f64) {
        self.values[self.head] = close;
        self.head = (self.head + 1) % self.values.len();
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// The `index`-th close of the window, `0` being the oldest.
    pub fn get(&self, index: usize) -> f64 {
        self.values[(self.head + index) % self.values.len()]
    }

    /// The mean of closes `from..to` of the window, NaN for an empty span, as
    /// the oracle's mean of an empty slice is.
    pub fn mean(&self, from: usize, to: usize) -> f64 {
        if to <= from {
            return f64::NAN;
        }
        (from..to).map(|index| self.get(index)).sum::<f64>() / (to - from) as f64
    }

    pub fn max(&self, from: usize, to: usize) -> f64 {
        (from..to)
            .map(|index| self.get(index))
            .fold(f64::NEG_INFINITY, f64::max)
    }

    pub fn min(&self, from: usize, to: usize) -> f64 {
        (from..to)
            .map(|index| self.get(index))
            .fold(f64::INFINITY, f64::min)
    }
}

/// A shape asked of the `period` closes before the bar and the bar's close.
pub type CloseRule<P> = fn(&Closes, f64, &P) -> f64;

/// Rounding tops and bottoms, scallops and V shapes: a rule over a trailing
/// window of closes, answering from bar `period` on.
#[derive(Clone, Debug)]
pub struct CloseState<P: Copy> {
    closes: Closes,
    params: P,
    rule: CloseRule<P>,
    bars: usize,
}

impl<P: Copy> CloseState<P> {
    pub fn new(period: usize, params: P, rule: CloseRule<P>) -> Self {
        Self {
            closes: Closes::new(period),
            params,
            rule,
            bars: 0,
        }
    }

    fn advance(&mut self, close: f64) -> Option<f64> {
        let now = self.bars;
        self.bars += 1;
        let value =
            (now >= self.closes.len()).then(|| (self.rule)(&self.closes, close, &self.params));
        self.closes.push(close);
        value
    }
}

impl<P: Copy + std::fmt::Debug> Step<3, 1> for CloseState<P> {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 1]> {
        self.advance(bar[2]).map(|value| [value])
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 1]> {
        self.clone().advance(bar[2]).map(|value| [value])
    }
}

/// The left, middle and right thirds' mean closes of a window, the way the
/// rounding and scallop shapes split it: `period / 3` closes, the next
/// `period / 3`, and the rest.
pub fn thirds(closes: &Closes) -> (f64, f64, f64) {
    let third = closes.len() / 3;
    (
        closes.mean(0, third),
        closes.mean(third, 2 * third),
        closes.mean(2 * third, closes.len()),
    )
}

/// What the diamond keeps of each bar's fit: whether both lines were drawn,
/// and their slopes.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Fit {
    fitted: bool,
    upper: f64,
    lower: f64,
}

const NO_FIT: Fit = Fit {
    fitted: false,
    upper: 0.0,
    lower: 0.0,
};

/// Diamond tops and bottoms: two trendlines over half the window that spread
/// apart half a window ago and now close in, and a close through one of them.
#[derive(Clone, Debug)]
pub struct DiamondState {
    bottom: bool,
    period: usize,
    swings: Swings,
    fits: Box<[Fit]>,
    head: usize,
    bars: usize,
}

impl DiamondState {
    pub fn new(bottom: bool, period: usize, pivot_n: usize) -> Self {
        let half = period / 2;
        Self {
            bottom,
            period,
            swings: Swings::new(pivot_n, half),
            fits: vec![NO_FIT; half + 1].into_boxed_slice(),
            head: 0,
            bars: 0,
        }
    }

    fn advance(&mut self, bar: [f64; 3]) -> Option<f64> {
        let now = self.bars;
        self.bars += 1;
        let confirmed = self.swings.find(now, bar[0], bar[1]);
        self.swings.file(now, confirmed);
        let lines = self.swings.lines(now);
        self.fits[self.head] = Fit {
            fitted: lines.fitted(),
            upper: lines.upper.slope,
            lower: lines.lower.slope,
        };
        self.head = (self.head + 1) % self.fits.len();
        if now < self.period {
            return None;
        }
        // The slot about to be overwritten holds the fit of `half` bars ago.
        let earlier = self.fits[self.head];
        let holds = lines.fitted()
            && earlier.fitted
            && earlier.upper > 0.0
            && earlier.lower < 0.0
            && lines.lower.slope > lines.upper.slope;
        let close = bar[2];
        Some(if self.bottom {
            if holds && close > lines.upper.level {
                100.0
            } else {
                0.0
            }
        } else if holds && close < lines.lower.level {
            -100.0
        } else {
            0.0
        })
    }
}

impl Step<3, 1> for DiamondState {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 1]> {
        self.advance(bar).map(|value| [value])
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 1]> {
        self.clone().advance(bar).map(|value| [value])
    }
}

/// Bump-and-run tops and bottoms: a lead-in line through one side's swing
/// points, a bump at least `factor` times steeper, and a close back through
/// the lead-in line extended to the current bar.
#[derive(Clone, Debug)]
pub struct BumpState {
    bottom: bool,
    lead: usize,
    bump: usize,
    factor: f64,
    finder: SwingFinder,
    log: SwingLog,
    bars: usize,
}

impl BumpState {
    pub fn new(bottom: bool, lead: usize, bump: usize, factor: f64, pivot_n: usize) -> Self {
        Self {
            bottom,
            lead,
            bump,
            factor,
            // A top's lines run through the swing lows, a bottom's through the
            // swing highs.
            finder: if bottom {
                SwingFinder::highs(pivot_n)
            } else {
                SwingFinder::lows(pivot_n)
            },
            log: SwingLog::new(lead + bump),
            bars: 0,
        }
    }

    pub fn lookback(lead: usize, bump: usize) -> usize {
        lead + bump
    }

    fn advance(&mut self, bar: [f64; 3]) -> Option<f64> {
        let now = self.bars;
        self.bars += 1;
        let side = if self.bottom { bar[0] } else { bar[1] };
        if let Some(price) = self.finder.push(side) {
            self.log.push(Swing { at: now, price });
        }
        self.log.trim(now);
        if now < Self::lookback(self.lead, self.bump) {
            return None;
        }
        // The lead-in window ends the bar before the bump's window starts, and
        // spans `lead` bars; the oracle's fit over it is read at this bar.
        let lead_end = now - self.bump - 1;
        let lead_start = lead_end + 1 - self.lead;
        let lead_in = self
            .log
            .iter()
            .filter(|s| s.at >= lead_start && s.at <= lead_end);
        let lead_line = Line::fit(lead_in, now);
        let bump = self.log.iter().filter(|s| s.at >= now - self.bump);
        let bump_line = Line::fit(bump, now);
        let close = bar[2];
        let steady = lead_line.points >= 2 && lead_line.r2 >= 0.6 && bump_line.points >= 2;
        Some(if self.bottom {
            let holds = steady
                && lead_line.slope < 0.0
                && bump_line.slope <= self.factor * lead_line.slope
                && close > lead_line.level;
            if holds { 100.0 } else { 0.0 }
        } else {
            let holds = steady
                && lead_line.slope > 0.0
                && bump_line.slope >= self.factor * lead_line.slope
                && close < lead_line.level;
            if holds { -100.0 } else { 0.0 }
        })
    }
}

impl Step<3, 1> for BumpState {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 1]> {
        self.advance(bar).map(|value| [value])
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 1]> {
        self.clone().advance(bar).map(|value| [value])
    }
}

/// The measured move up (and down): two consecutive swing highs (lows), each
/// the end of a leg from the swing low (high) before it, legs within
/// `leg_tol` of each other, read on the bar that confirms the second.
///
/// Unlike the window shapes it remembers only the swing points it needs: the
/// last swing of each side and the most extreme opposite swing since the last
/// main one.
#[derive(Clone, Debug)]
pub struct MeasuredMoveState {
    down: bool,
    leg_tol: f64,
    highs: SwingFinder,
    lows: SwingFinder,
    /// The previous main swing's price, with the opposite swing that started
    /// its leg: the latest one confirmed before it.
    previous: Option<(f64, Option<f64>)>,
    /// The latest opposite swing confirmed so far.
    last_other: Option<f64>,
    /// The most extreme opposite swing confirmed since the previous main swing.
    extreme_since: Option<f64>,
    lookback: usize,
    bars: usize,
}

impl MeasuredMoveState {
    pub fn new(down: bool, pivot_n: usize, leg_tol: f64) -> Self {
        Self {
            down,
            leg_tol,
            highs: SwingFinder::highs(pivot_n),
            lows: SwingFinder::lows(pivot_n),
            previous: None,
            last_other: None,
            extreme_since: None,
            lookback: Self::lookback(pivot_n),
            bars: 0,
        }
    }

    /// The earliest bar a second main swing can be confirmed on: a swing of
    /// each side on the first two bars, `pivot_n` bars on, then one more of
    /// each.
    pub fn lookback(pivot_n: usize) -> usize {
        pivot_n + 3
    }

    fn advance(&mut self, bar: [f64; 3]) -> Option<f64> {
        let now = self.bars;
        self.bars += 1;
        let high = self.highs.push(bar[0]);
        let low = self.lows.push(bar[1]);
        let (main, other) = if self.down { (low, high) } else { (high, low) };
        let mut value = 0.0;
        // A main swing first: an opposite swing confirmed on the same bar is
        // not between the two, nor before the first.
        if let Some(price) = main {
            if let (Some((first, Some(first_start))), Some(second_start)) =
                (self.previous, self.extreme_since)
            {
                let (leg1, leg2) = if self.down {
                    (first_start - first, second_start - price)
                } else {
                    (first - first_start, price - second_start)
                };
                let close = bar[2];
                let beyond = if self.down {
                    close < second_start
                } else {
                    close > first
                };
                if leg1 > 0.0
                    && leg2 > 0.0
                    && (leg2 - leg1).abs() / (leg1 + crate::core::chart::EPS) < self.leg_tol
                    && beyond
                {
                    value = if self.down { -100.0 } else { 100.0 };
                }
            }
            self.previous = Some((price, self.last_other));
            self.extreme_since = None;
        }
        if let Some(price) = other {
            self.last_other = Some(price);
            // Between two main swings means strictly after the first: one
            // confirmed on the first's own bar starts no leg.
            if main.is_none() {
                self.extreme_since = Some(match self.extreme_since {
                    None => price,
                    Some(best) if self.down => best.max(price),
                    Some(best) => best.min(price),
                });
            }
        }
        (now >= self.lookback).then_some(value)
    }
}

impl Step<3, 1> for MeasuredMoveState {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 1]> {
        self.advance(bar).map(|value| [value])
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 1]> {
        self.clone().advance(bar).map(|value| [value])
    }
}

/// Three peaks (valleys): the last three swing highs (lows) confirmed in the
/// window, each lower (higher) than the one before and `min_separation` bars
/// apart, and a close below the lowest low (above the highest high) of the bars
/// from the first one's confirmation to the third's.
#[derive(Clone, Debug)]
pub struct StairState {
    valleys: bool,
    period: usize,
    min_separation: usize,
    finder: SwingFinder,
    main: crate::core::chart::ClusteredLog,
    /// The opposite side's value for each of the last `period + 1` bars.
    opposite: Box<[f64]>,
    bars: usize,
}

impl StairState {
    pub fn new(valleys: bool, period: usize, pivot_n: usize, min_separation: usize) -> Self {
        Self {
            valleys,
            period,
            min_separation,
            finder: if valleys {
                SwingFinder::lows(pivot_n)
            } else {
                SwingFinder::highs(pivot_n)
            },
            main: crate::core::chart::ClusteredLog::new(pivot_n, period),
            opposite: vec![0.0; period + 1].into_boxed_slice(),
            bars: 0,
        }
    }

    fn opposite_at(&self, bar: usize) -> f64 {
        self.opposite[bar % self.opposite.len()]
    }

    /// Read before this bar's own swing point is filed.
    fn read(&self, now: usize, close: f64) -> f64 {
        let oldest = now - self.period;
        let inside = self.main.log.iter().filter(|s| s.at >= oldest);
        let count = inside.clone().count();
        if count < 3 {
            return 0.0;
        }
        let mut last = inside.skip(count - 3);
        let (Some(first), Some(second), Some(third)) = (last.next(), last.next(), last.next())
        else {
            return 0.0;
        };
        if second.at - first.at < self.min_separation || third.at - second.at < self.min_separation
        {
            return 0.0;
        }
        let span = (first.at..=third.at).map(|bar| self.opposite_at(bar));
        if self.valleys {
            let resistance = span.fold(f64::NEG_INFINITY, f64::max);
            let rising = first.price < second.price && second.price < third.price;
            if rising && close > resistance {
                100.0
            } else {
                0.0
            }
        } else {
            let support = span.fold(f64::INFINITY, f64::min);
            let falling = first.price > second.price && second.price > third.price;
            if falling && close < support {
                -100.0
            } else {
                0.0
            }
        }
    }

    fn advance(&mut self, bar: [f64; 3]) -> Option<f64> {
        let now = self.bars;
        self.bars += 1;
        let value = (now >= self.period).then(|| self.read(now, bar[2]));
        let (main, opposite) = if self.valleys {
            (bar[1], bar[0])
        } else {
            (bar[0], bar[1])
        };
        let len = self.opposite.len();
        self.opposite[now % len] = opposite;
        if let Some(price) = self.finder.push(main) {
            self.main.push(Swing { at: now, price });
        }
        self.main.log.trim(now);
        value
    }
}

impl Step<3, 1> for StairState {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 1]> {
        self.advance(bar).map(|value| [value])
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 1]> {
        self.clone().advance(bar).map(|value| [value])
    }
}
