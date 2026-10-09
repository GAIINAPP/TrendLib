//! What the chart patterns are written in terms of.
//!
//! A chart pattern is a shape drawn through swing points: bars at least as high
//! (or as low) as the `pivot_n` bars on either side of them. A swing point is
//! only known `pivot_n` bars after the fact, so it is filed under the bar that
//! confirmed it, and that is also where it sits when a line is drawn through
//! it. Both are the conventions the reference implementation uses, which is
//! what lets its numbers check these.
//!
//! Everything a pattern reads is held in windows of a fixed length, so a stream
//! carries no more than a batch run does and both call the same `push`.

use std::collections::VecDeque;

use crate::core::error::TlError;

/// The oracle's guard against dividing by a price of zero. It is part of every
/// comparison the oracle makes, so it is part of these too.
pub const EPS: f64 = 1e-10;

/// Refuses a whole-numbered parameter outside its range.
pub fn whole(
    indicator: &str,
    param: &str,
    value: usize,
    min: usize,
    max: usize,
) -> Result<(), TlError> {
    if (min..=max).contains(&value) {
        Ok(())
    } else {
        Err(TlError::param_out_of_range(
            indicator, param, value, min, max,
        ))
    }
}

/// Refuses a tolerance below its floor, or one that is not a finite number.
pub fn at_least(indicator: &str, param: &str, value: f64, min: f64) -> Result<(), TlError> {
    if value.is_finite() && value >= min {
        Ok(())
    } else {
        Err(TlError::float_param_out_of_range(
            indicator,
            param,
            value,
            min,
            f64::INFINITY,
        ))
    }
}

/// A swing point: the bar that confirmed it, counted from the first valid bar,
/// and the price of the extreme it confirmed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Swing {
    pub at: usize,
    pub price: f64,
}

/// Finds the swing highs (or lows) of one series.
///
/// Bar `t - n` is a swing high when no bar from `t - 2n` to `t` is higher, and
/// that is known at bar `t`. Near the start the window holds only the bars there
/// are, which is how the oracle pads it, so the first bar can be a swing point.
#[derive(Clone, Debug)]
pub struct SwingFinder {
    n: usize,
    highs: bool,
    values: Box<[f64]>,
    head: usize,
    seen: usize,
}

impl SwingFinder {
    pub fn highs(n: usize) -> Self {
        Self::new(n, true)
    }

    pub fn lows(n: usize) -> Self {
        Self::new(n, false)
    }

    fn new(n: usize, highs: bool) -> Self {
        Self {
            n,
            highs,
            values: vec![0.0; 2 * n + 1].into_boxed_slice(),
            head: 0,
            seen: 0,
        }
    }

    fn back(&self, back: usize) -> f64 {
        let len = self.values.len();
        self.values[(self.head + len - 1 - back) % len]
    }

    /// Takes the next bar's value and returns the price of the swing point it
    /// confirms, if it confirms one.
    pub fn push(&mut self, value: f64) -> Option<f64> {
        self.values[self.head] = value;
        self.head = (self.head + 1) % self.values.len();
        self.seen += 1;
        if self.seen <= self.n {
            return None;
        }
        let candidate = self.back(self.n);
        let span = self.seen.min(self.values.len());
        let beaten = (0..span).map(|back| self.back(back)).any(|other| {
            if self.highs {
                other > candidate
            } else {
                other < candidate
            }
        });
        (!beaten).then_some(candidate)
    }
}

/// Swing points confirmed within a trailing window, oldest first.
///
/// Sized when it is made: a window of `period` bars confirms at most one swing
/// point a bar, so it never has to grow.
#[derive(Clone, Debug)]
pub struct SwingLog {
    period: usize,
    items: VecDeque<Swing>,
}

impl SwingLog {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            items: VecDeque::with_capacity(period + 2),
        }
    }

    pub fn push(&mut self, swing: Swing) {
        self.items.push_back(swing);
    }

    /// Forgets swing points confirmed before the window that ends at `now`.
    pub fn trim(&mut self, now: usize) {
        let oldest = now.saturating_sub(self.period);
        while self.items.front().is_some_and(|swing| swing.at < oldest) {
            self.items.pop_front();
        }
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = Swing> + Clone + '_ {
        self.items.iter().copied()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// The `back`-th newest swing point, `0` being the newest.
    pub fn newest(&self, back: usize) -> Option<Swing> {
        self.items
            .len()
            .checked_sub(back + 1)
            .and_then(|index| self.items.get(index).copied())
    }
}

/// A least-squares line through a window's swing points, read at the current
/// bar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line {
    /// Price per bar.
    pub slope: f64,
    /// Where the line is at the current bar.
    pub level: f64,
    /// How much of the swing points' spread the line accounts for, 0 to 1.
    pub r2: f64,
    pub points: usize,
}

impl Line {
    /// Fits a line through `swings`, evaluated at bar `now`.
    ///
    /// The sums are taken afresh from the window, in bars counted back from
    /// `now`, rather than as differences of running totals: two totals of the
    /// same size subtracted lose most of their digits, and what is left would
    /// decide whether a line rises (`CONVENTIONS.md` section 1, deviation 9).
    pub fn fit(swings: impl Iterator<Item = Swing> + Clone, now: usize) -> Self {
        let points = swings.clone().count();
        if points < 2 {
            // The oracle's answer for a single point: its own price, no fit.
            let level = swings.map(|s| s.price).next().unwrap_or(0.0);
            return Self {
                slope: 0.0,
                level,
                r2: 0.0,
                points,
            };
        }

        // A window whose prices are all equal is a flat line through them,
        // exactly. Averaging would leave a residue of a unit in the last place,
        // and its sign would decide whether the line rises.
        let first = swings.clone().map(|s| s.price).next().unwrap_or(0.0);
        if swings.clone().all(|s| s.price == first) {
            return Self {
                slope: 0.0,
                level: first,
                r2: 1.0,
                points,
            };
        }

        let count = points as f64;
        let x = |s: Swing| s.at as f64 - now as f64;
        let mean_x = swings.clone().map(x).sum::<f64>() / count;
        let mean_y = swings.clone().map(|s| s.price).sum::<f64>() / count;
        let mut ss_xx = 0.0;
        let mut ss_xy = 0.0;
        let mut ss_yy = 0.0;
        for swing in swings {
            let dx = x(swing) - mean_x;
            let dy = swing.price - mean_y;
            ss_xx += dx * dx;
            ss_xy += dx * dy;
            ss_yy += dy * dy;
        }
        if ss_xx <= EPS {
            return Self {
                slope: 0.0,
                level: mean_y,
                r2: 0.0,
                points,
            };
        }
        let slope = ss_xy / ss_xx;
        let r2 = if ss_yy > EPS {
            1.0 - (ss_yy - slope * ss_xy) / ss_yy
        } else {
            1.0
        };
        Self {
            slope,
            level: mean_y - slope * mean_x,
            r2,
            points,
        }
    }
}

/// The two lines a trendline pattern is drawn with: one through the swing
/// highs, one through the swing lows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lines {
    pub upper: Line,
    pub lower: Line,
}

impl Lines {
    /// Both lines exist and both account for at least half of their points'
    /// spread, which is the oracle's test that there are lines here at all.
    pub fn fitted(&self) -> bool {
        self.upper.points >= 2
            && self.lower.points >= 2
            && self.upper.r2 >= 0.5
            && self.lower.r2 >= 0.5
    }

    /// The gap between the lines is closing: the lower one climbs faster than
    /// the upper one.
    pub fn converging(&self) -> bool {
        self.lower.slope > self.upper.slope
    }

    /// `+100` for a close above the upper line, `-100` for one below the lower
    /// line, `0` between them. Upward is asked first, which is the order the
    /// oracle's two-sided patterns ask in.
    pub fn breakout(&self, close: f64) -> f64 {
        if close > self.upper.level {
            100.0
        } else if close < self.lower.level {
            -100.0
        } else {
            0.0
        }
    }
}

/// Swing points on both sides of a series, kept for a trailing window.
#[derive(Clone, Debug)]
pub struct Swings {
    highs: SwingFinder,
    lows: SwingFinder,
    pub upper: SwingLog,
    pub lower: SwingLog,
}

/// Swing points confirmed by the bar just pushed.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Confirmed {
    pub high: Option<Swing>,
    pub low: Option<Swing>,
}

impl Swings {
    pub fn new(pivot_n: usize, period: usize) -> Self {
        Self {
            highs: SwingFinder::highs(pivot_n),
            lows: SwingFinder::lows(pivot_n),
            upper: SwingLog::new(period),
            lower: SwingLog::new(period),
        }
    }

    /// Finds what bar `now` confirms without filing it.
    pub fn find(&mut self, now: usize, high: f64, low: f64) -> Confirmed {
        Confirmed {
            high: self.highs.push(high).map(|price| Swing { at: now, price }),
            low: self.lows.push(low).map(|price| Swing { at: now, price }),
        }
    }

    /// Files what bar `now` confirmed and forgets what has left the window.
    pub fn file(&mut self, now: usize, confirmed: Confirmed) {
        if let Some(swing) = confirmed.high {
            self.upper.push(swing);
        }
        if let Some(swing) = confirmed.low {
            self.lower.push(swing);
        }
        self.upper.trim(now);
        self.lower.trim(now);
    }

    pub fn lines(&self, now: usize) -> Lines {
        Lines {
            upper: Line::fit(self.upper.iter(), now),
            lower: Line::fit(self.lower.iter(), now),
        }
    }
}

/// How a trendline pattern reads the lines and the close at one bar: `+100`,
/// `-100` or `0`. The `f64` is the pattern's tolerance, where it has one.
pub type TrendlineRule = fn(&Lines, f64, f64) -> f64;

/// Triangles, wedges, rectangles, channels and the broadening formation: a
/// line through the swing highs of the last `period` bars, another through the
/// swing lows, and a rule about their slopes and where the close is.
#[derive(Clone, Debug)]
pub struct TrendlineState {
    swings: Swings,
    period: usize,
    tolerance: f64,
    rule: TrendlineRule,
    bars: usize,
}

impl TrendlineState {
    pub fn new(pivot_n: usize, period: usize, tolerance: f64, rule: TrendlineRule) -> Self {
        Self {
            swings: Swings::new(pivot_n, period),
            period,
            tolerance,
            rule,
            bars: 0,
        }
    }

    fn advance(&mut self, bar: [f64; 3]) -> Option<f64> {
        let now = self.bars;
        self.bars += 1;
        let confirmed = self.swings.find(now, bar[0], bar[1]);
        self.swings.file(now, confirmed);
        if now < self.period {
            return None;
        }
        let lines = self.swings.lines(now);
        if !lines.fitted() {
            return Some(0.0);
        }
        Some((self.rule)(&lines, bar[2], self.tolerance))
    }
}

impl crate::core::kernel::Step<3, 1> for TrendlineState {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 1]> {
        self.advance(bar).map(|value| [value])
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 1]> {
        self.clone().advance(bar).map(|value| [value])
    }
}

/// What a flag or pennant reads at one bar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pole {
    /// `+1` when the pole rose by at least `min_pole`, `-1` when it fell by at
    /// least that much, `0` otherwise. Rising is asked first.
    pub direction: i8,
    /// The close where the pole starts, `pole_bars` before it ends.
    pub base: f64,
    /// The close `period` bars ago, where the flag starts.
    pub start: f64,
    pub close: f64,
    pub lines: Lines,
}

impl Pole {
    /// How far the flag has run back over the pole, as a fraction of the pole:
    /// the flag's move over the pole's, both measured to the flag's start.
    pub fn retrace(&self) -> f64 {
        let flag = (self.close - self.start).abs();
        let pole = if self.direction > 0 {
            self.start - self.base
        } else {
            self.base - self.start
        };
        flag / (pole + EPS)
    }
}

/// How a flag or pennant reads one bar. The `f64` is `max_retrace`.
pub type PoleRule = fn(&Pole, f64) -> f64;

/// Flags and pennants: a sharp move of `pole_bars` bars ending just before the
/// last `period` bars, then a consolidation drawn with two trendlines over
/// those `period` bars.
#[derive(Clone, Debug)]
pub struct PoleState {
    swings: Swings,
    closes: Box<[f64]>,
    head: usize,
    period: usize,
    pole_bars: usize,
    min_pole: f64,
    max_retrace: f64,
    rule: PoleRule,
    bars: usize,
}

impl PoleState {
    pub fn new(
        pivot_n: usize,
        period: usize,
        pole_bars: usize,
        min_pole: f64,
        max_retrace: f64,
        rule: PoleRule,
    ) -> Self {
        Self {
            swings: Swings::new(pivot_n, period),
            closes: vec![0.0; pole_bars + period + 2].into_boxed_slice(),
            head: 0,
            period,
            pole_bars,
            min_pole,
            max_retrace,
            rule,
            bars: 0,
        }
    }

    /// The first bar with a whole pole behind it. The oracle starts a bar
    /// earlier, where the pole would begin before the first bar, and reads
    /// the input's last bar in its place (`CONVENTIONS.md` deviation 8).
    pub fn lookback(period: usize, pole_bars: usize) -> usize {
        pole_bars + period + 1
    }

    fn close_back(&self, back: usize) -> f64 {
        let len = self.closes.len();
        self.closes[(self.head + len - 1 - back) % len]
    }

    fn advance(&mut self, bar: [f64; 3]) -> Option<f64> {
        let now = self.bars;
        self.bars += 1;
        self.closes[self.head] = bar[2];
        self.head = (self.head + 1) % self.closes.len();
        let confirmed = self.swings.find(now, bar[0], bar[1]);
        self.swings.file(now, confirmed);
        if now < Self::lookback(self.period, self.pole_bars) {
            return None;
        }

        let start = self.close_back(self.period);
        let top = self.close_back(self.period + 1);
        let base = self.close_back(self.period + 1 + self.pole_bars);
        let moved = (top - base) / (base + EPS);
        let direction = if moved >= self.min_pole {
            1
        } else if moved <= -self.min_pole {
            -1
        } else {
            0
        };
        if direction == 0 {
            return Some(0.0);
        }
        let lines = self.swings.lines(now);
        if !lines.fitted() {
            return Some(0.0);
        }
        let pole = Pole {
            direction,
            base,
            start,
            close: bar[2],
            lines,
        };
        Some((self.rule)(&pole, self.max_retrace))
    }
}

impl crate::core::kernel::Step<3, 1> for PoleState {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 1]> {
        self.advance(bar).map(|value| [value])
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 1]> {
        self.clone().advance(bar).map(|value| [value])
    }
}

/// Whether two prices are within `tol` of each other, as a fraction of the
/// larger of the two.
pub fn near(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() / (a.abs().max(b.abs()) + EPS) <= tol
}

/// Swing points with the repeats of one flat extreme folded into its first.
///
/// Two swing highs confirmed within `pivot_n` bars of each other must be equal,
/// since each is at least as high as the other, and a run of them is one
/// extreme seen several times. Only the first is kept, as the oracle keeps it,
/// so a top is never compared with its own echo.
#[derive(Clone, Debug)]
pub struct ClusteredLog {
    pivot_n: usize,
    last_raw: Option<usize>,
    pub log: SwingLog,
}

impl ClusteredLog {
    pub fn new(pivot_n: usize, period: usize) -> Self {
        Self {
            pivot_n,
            last_raw: None,
            log: SwingLog::new(period),
        }
    }

    /// Files a swing point unless it repeats the last one, and says which.
    pub fn push(&mut self, swing: Swing) -> bool {
        let repeat = self
            .last_raw
            .is_some_and(|last| swing.at - last <= self.pivot_n);
        self.last_raw = Some(swing.at);
        if !repeat {
            self.log.push(swing);
        }
        !repeat
    }

    /// The newest swing point confirmed before bar `before`.
    pub fn last_before(&self, before: usize) -> Option<Swing> {
        self.log.iter().rev().find(|swing| swing.at < before)
    }
}

/// Double and triple tops and bottoms: `count` swing extremes at about the same
/// price inside the window, separated by at least `min_separation` bars, and a
/// close beyond the neckline the swings between them draw.
#[derive(Clone, Debug)]
pub struct RepeatState {
    count: usize,
    top: bool,
    period: usize,
    tol: f64,
    min_separation: usize,
    highs: SwingFinder,
    lows: SwingFinder,
    main: ClusteredLog,
    other: ClusteredLog,
    shapes: Option<Shapes>,
    bars: usize,
}

/// Bulkowski's Adam (a sharp, narrow extreme) and Eve (a rounded, wide one), as
/// the oracle grades them: over the extreme's bar and the three either side,
/// how far the mean sits from the extreme, as a share of the window's range.
/// A grade of at least 0.37 is Adam.
#[derive(Clone, Debug)]
struct Shapes {
    /// The two tops' (bottoms') required shapes, oldest first: `Some(true)` for
    /// Adam, `Some(false)` for Eve.
    wanted: [bool; 2],
    pivot_n: usize,
    /// The main side's last `pivot_n + 4` prices, newest last.
    recent: Box<[f64]>,
    head: usize,
    seen: usize,
    /// Each filed main swing's grade, by the bar that confirmed it.
    adam: VecDeque<(usize, bool)>,
}

impl Shapes {
    const ADAM: f64 = 0.37;

    fn push(&mut self, value: f64) {
        self.recent[self.head] = value;
        self.head = (self.head + 1) % self.recent.len();
        self.seen += 1;
    }

    /// The price `back` bars before the newest.
    fn back(&self, back: usize) -> f64 {
        let len = self.recent.len();
        self.recent[(self.head + len - 1 - back) % len]
    }

    /// Whether the extreme `pivot_n` bars back is an Adam, read when its swing
    /// is confirmed. Its three bars on the right are already in, since
    /// `pivot_n` is at least 3; on the left the window holds the bars there are.
    fn is_adam(&self, top: bool) -> bool {
        let newest = self.pivot_n - 3;
        let oldest = (self.pivot_n + 3).min(self.seen - 1);
        let window = (newest..=oldest).map(|back| self.back(back));
        let count = (oldest - newest + 1) as f64;
        let high = window.clone().fold(f64::NEG_INFINITY, f64::max);
        let low = window.clone().fold(f64::INFINITY, f64::min);
        let range = high - low;
        if range <= 0.0 {
            return 0.0 >= Self::ADAM;
        }
        let mean = window.sum::<f64>() / count;
        let grade = if top {
            (high - mean) / range
        } else {
            (mean - low) / range
        };
        grade.clamp(0.0, 1.0) >= Self::ADAM
    }

    fn grade_of(&self, at: usize) -> Option<bool> {
        self.adam
            .iter()
            .find(|(when, _)| *when == at)
            .map(|(_, adam)| *adam)
    }
}

impl RepeatState {
    pub fn new(
        count: usize,
        top: bool,
        pivot_n: usize,
        period: usize,
        tol: f64,
        min_separation: usize,
    ) -> Self {
        Self {
            count,
            top,
            period,
            tol,
            min_separation,
            highs: SwingFinder::highs(pivot_n),
            lows: SwingFinder::lows(pivot_n),
            main: ClusteredLog::new(pivot_n, period),
            other: ClusteredLog::new(pivot_n, period),
            shapes: None,
            bars: 0,
        }
    }

    /// A double top (bottom) whose two extremes must be Adam (`true`) or Eve
    /// (`false`), oldest first. `pivot_n` must be at least 3.
    pub fn with_shapes(mut self, first: bool, second: bool, pivot_n: usize) -> Self {
        self.shapes = Some(Shapes {
            wanted: [first, second],
            pivot_n,
            recent: vec![0.0; pivot_n + 4].into_boxed_slice(),
            head: 0,
            seen: 0,
            adam: VecDeque::with_capacity(self.period + 2),
        });
        self
    }

    /// Read before this bar's own swing points are filed: the oracle counts
    /// only those confirmed by an earlier bar.
    fn read(&self, now: usize, close: f64) -> f64 {
        let mut peaks = [Swing { at: 0, price: 0.0 }; 3];
        for (slot, back) in (0..self.count).rev().enumerate() {
            match self.main.log.newest(back) {
                Some(swing) => peaks[slot] = swing,
                None => return 0.0,
            }
        }
        let peaks = &peaks[..self.count];
        if peaks[0].at < now - self.period {
            return 0.0;
        }
        if let Some(shapes) = &self.shapes {
            for (peak, wanted) in peaks.iter().zip(shapes.wanted) {
                if shapes.grade_of(peak.at) != Some(wanted) {
                    return 0.0;
                }
            }
        }
        let mut neckline: Option<f64> = None;
        for pair in peaks.windows(2) {
            let (earlier, later) = (pair[0], pair[1]);
            if later.at - earlier.at < self.min_separation
                || !near(earlier.price, later.price, self.tol)
            {
                return 0.0;
            }
            let Some(between) = self.other.last_before(later.at) else {
                return 0.0;
            };
            if between.at <= earlier.at {
                return 0.0;
            }
            neckline = Some(match neckline {
                None => between.price,
                Some(level) if self.top => level.min(between.price),
                Some(level) => level.max(between.price),
            });
        }
        match neckline {
            Some(level) if self.top && close < level => -100.0,
            Some(level) if !self.top && close > level => 100.0,
            _ => 0.0,
        }
    }

    fn advance(&mut self, bar: [f64; 3]) -> Option<f64> {
        let now = self.bars;
        self.bars += 1;
        let high = self
            .highs
            .push(bar[0])
            .map(|price| Swing { at: now, price });
        let low = self.lows.push(bar[1]).map(|price| Swing { at: now, price });
        let value = (now >= self.period).then(|| self.read(now, bar[2]));
        let (main, other) = if self.top { (high, low) } else { (low, high) };
        if let Some(shapes) = &mut self.shapes {
            shapes.push(if self.top { bar[0] } else { bar[1] });
        }
        if let Some(swing) = main
            && self.main.push(swing)
            && let Some(shapes) = &mut self.shapes
        {
            let adam = shapes.is_adam(self.top);
            shapes.adam.push_back((swing.at, adam));
        }
        if let Some(swing) = other {
            self.other.push(swing);
        }
        self.main.log.trim(now);
        self.other.log.trim(now);
        if let Some(shapes) = &mut self.shapes {
            let oldest = now.saturating_sub(self.period);
            while shapes.adam.front().is_some_and(|(at, _)| *at < oldest) {
                shapes.adam.pop_front();
            }
        }
        value
    }
}

impl crate::core::kernel::Step<3, 1> for RepeatState {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 1]> {
        self.advance(bar).map(|value| [value])
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 1]> {
        self.clone().advance(bar).map(|value| [value])
    }
}

/// A neckline waiting for a close through it.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Neckline {
    /// The bar that confirmed the right shoulder.
    armed_at: usize,
    left: Swing,
    right: Swing,
}

impl Neckline {
    fn level(&self, now: usize) -> f64 {
        let run = self.right.at as f64 - self.left.at as f64;
        let slope = (self.right.price - self.left.price) / run;
        self.left.price + slope * (now as f64 - self.left.at as f64)
    }
}

/// Head and shoulders, and the inverse: a head beyond two shoulders of about the
/// same height, and the first close through the neckline drawn under (over) the
/// two swings between them within `period` bars of the right shoulder.
#[derive(Clone, Debug)]
pub struct ShouldersState {
    top: bool,
    period: usize,
    shoulder_tol: f64,
    min_separation: usize,
    highs: SwingFinder,
    lows: SwingFinder,
    main: SwingLog,
    other: SwingLog,
    armed: VecDeque<Neckline>,
    lookback: usize,
    bars: usize,
}

impl ShouldersState {
    pub fn new(
        top: bool,
        pivot_n: usize,
        period: usize,
        shoulder_tol: f64,
        min_separation: usize,
    ) -> Self {
        Self {
            top,
            period,
            shoulder_tol,
            min_separation,
            highs: SwingFinder::highs(pivot_n),
            lows: SwingFinder::lows(pivot_n),
            main: SwingLog::new(period),
            other: SwingLog::new(period),
            armed: VecDeque::with_capacity(period + 2),
            lookback: Self::lookback(pivot_n, min_separation),
            bars: 0,
        }
    }

    /// The earliest bar a neckline can be crossed on: the left shoulder
    /// confirmed `pivot_n` bars in, the head and the right shoulder at least
    /// `min_separation` apart after it, and the close one bar after that.
    pub fn lookback(pivot_n: usize, min_separation: usize) -> usize {
        pivot_n + 2 * min_separation + 1
    }

    /// Further from the neckline than the shoulders: higher for a top, lower
    /// for the inverse.
    fn beyond(&self, a: f64, b: f64) -> bool {
        if self.top { a > b } else { a < b }
    }

    fn shoulders_match(&self, left: f64, right: f64) -> bool {
        // The oracle measures a top against the higher shoulder and the inverse
        // against the larger magnitude; for positive prices they agree.
        let scale = if self.top {
            left.max(right)
        } else {
            left.abs().max(right.abs())
        };
        (left - right).abs() / (scale + EPS) <= self.shoulder_tol
    }

    /// The neckline a newly confirmed right shoulder completes, if it does.
    fn complete(&self, right: Swing) -> Option<Neckline> {
        let oldest = right.at.saturating_sub(self.period);
        let earlier = self.main.iter().filter(|s| s.at >= oldest);
        if earlier.clone().count() < 2 {
            return None;
        }
        // The head is the most extreme earlier swing, the first one if two tie.
        let (head_index, head) = earlier.clone().enumerate().fold(
            (0, earlier.clone().next()?),
            |(best_index, best), (index, swing)| {
                if self.beyond(swing.price, best.price) {
                    (index, swing)
                } else {
                    (best_index, best)
                }
            },
        );
        if !self.beyond(head.price, right.price) {
            return None;
        }
        let left = earlier
            .take(head_index)
            .filter(|s| head.at - s.at >= self.min_separation)
            .last()?;
        if right.at - head.at < self.min_separation
            || !self.shoulders_match(left.price, right.price)
        {
            return None;
        }
        let first = self
            .other
            .iter()
            .rev()
            .find(|s| s.at > left.at && s.at < head.at)?;
        let second = self
            .other
            .iter()
            .rev()
            .find(|s| s.at > head.at && s.at < right.at)?;
        Some(Neckline {
            armed_at: right.at,
            left: first,
            right: second,
        })
    }

    fn advance(&mut self, bar: [f64; 3]) -> Option<f64> {
        let now = self.bars;
        self.bars += 1;
        let close = bar[2];

        // Each neckline waits `period` bars after its right shoulder and is
        // spent by the first close through it.
        let (top, period) = (self.top, self.period);
        let mut crossed = false;
        self.armed.retain(|neckline| {
            if now >= neckline.armed_at + period {
                return false;
            }
            let level = neckline.level(now);
            let through = if top { close < level } else { close > level };
            crossed |= through;
            !through
        });

        let high = self
            .highs
            .push(bar[0])
            .map(|price| Swing { at: now, price });
        let low = self.lows.push(bar[1]).map(|price| Swing { at: now, price });
        let (main, other) = if self.top { (high, low) } else { (low, high) };
        if let Some(right) = main
            && let Some(neckline) = self.complete(right)
        {
            self.armed.push_back(neckline);
        }
        if let Some(swing) = main {
            self.main.push(swing);
        }
        if let Some(swing) = other {
            self.other.push(swing);
        }
        self.main.trim(now);
        self.other.trim(now);

        if now < self.lookback {
            return None;
        }
        Some(match (crossed, self.top) {
            (true, true) => -100.0,
            (true, false) => 100.0,
            (false, _) => 0.0,
        })
    }
}

impl crate::core::kernel::Step<3, 1> for ShouldersState {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 1]> {
        self.advance(bar).map(|value| [value])
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 1]> {
        self.clone().advance(bar).map(|value| [value])
    }
}

/// Cup with handle, and the inverted one: a rim and a floor drawn from the swing
/// points of the cup, the closes of the handle that follows it, and a close
/// beyond the rim.
///
/// The cup is the `cup` bars before the handle's `handle` bars, both ending just
/// before the current one. Its swing points are filed by the bar that confirmed
/// them, so a swing confirmed during the handle belongs to the handle.
#[derive(Clone, Debug)]
pub struct CupState {
    inverted: bool,
    cup: usize,
    handle: usize,
    max_retrace: f64,
    swings: Swings,
    closes: Box<[f64]>,
    head: usize,
    bars: usize,
}

impl CupState {
    pub fn new(
        inverted: bool,
        cup: usize,
        handle: usize,
        pivot_n: usize,
        max_retrace: f64,
    ) -> Self {
        Self {
            inverted,
            cup,
            handle,
            max_retrace,
            swings: Swings::new(pivot_n, cup + handle),
            closes: vec![0.0; handle].into_boxed_slice(),
            head: 0,
            bars: 0,
        }
    }

    /// The first bar with a whole cup and handle behind it.
    pub fn lookback(cup: usize, handle: usize) -> usize {
        cup + handle
    }

    /// Read before this bar's close joins the handle.
    fn read(&self, now: usize, close: f64) -> f64 {
        let start = now - self.cup - self.handle;
        let end = now - self.handle;
        let inside = |s: &Swing| s.at >= start && s.at < end;
        let highest = self
            .swings
            .upper
            .iter()
            .filter(inside)
            .map(|s| s.price)
            .reduce(f64::max);
        let lowest = self
            .swings
            .lower
            .iter()
            .filter(inside)
            .map(|s| s.price)
            .reduce(f64::min);
        let (Some(highest), Some(lowest)) = (highest, lowest) else {
            return 0.0;
        };
        let depth = highest - lowest;
        if self.inverted {
            let rim = lowest;
            let handle_low = self.closes.iter().copied().fold(f64::INFINITY, f64::min);
            let holds = depth / (highest + EPS) >= 0.05
                && handle_low <= rim * 1.05
                && (close - handle_low) / (depth + EPS) <= self.max_retrace
                && close < rim;
            if holds { -100.0 } else { 0.0 }
        } else {
            let rim = highest;
            let handle_high = self
                .closes
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max);
            let holds = depth / (rim + EPS) >= 0.05
                && handle_high >= rim * 0.95
                && (handle_high - close) / (depth + EPS) <= self.max_retrace
                && close > rim;
            if holds { 100.0 } else { 0.0 }
        }
    }

    fn advance(&mut self, bar: [f64; 3]) -> Option<f64> {
        let now = self.bars;
        self.bars += 1;
        let confirmed = self.swings.find(now, bar[0], bar[1]);
        self.swings.file(now, confirmed);
        let value = (now >= Self::lookback(self.cup, self.handle)).then(|| self.read(now, bar[2]));
        if !self.closes.is_empty() {
            self.closes[self.head] = bar[2];
            self.head = (self.head + 1) % self.closes.len();
        }
        value
    }
}

impl crate::core::kernel::Step<3, 1> for CupState {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 1]> {
        self.advance(bar).map(|value| [value])
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 1]> {
        self.clone().advance(bar).map(|value| [value])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn swings(points: &[(usize, f64)]) -> Vec<Swing> {
        points
            .iter()
            .map(|&(at, price)| Swing { at, price })
            .collect()
    }

    #[test]
    fn swing_points_of_one_price_draw_an_exactly_flat_line() {
        // Deviation 9: averaging would leave a residue whose sign decided
        // whether the line rises.
        let points = swings(&[(3, 100.1), (11, 100.1), (19, 100.1), (27, 100.1)]);
        let line = Line::fit(points.iter().copied(), 30);
        assert_eq!(line.slope, 0.0);
        assert_eq!(line.level, 100.1);
        assert_eq!(line.r2, 1.0);
        assert_eq!(line.points, 4);
    }

    #[test]
    fn a_line_through_collinear_points_is_reproduced() {
        let points = swings(&[(0, 2.0), (2, 3.0), (4, 4.0), (6, 5.0)]);
        let line = Line::fit(points.iter().copied(), 10);
        assert_eq!(line.slope, 0.5);
        assert_eq!(line.level, 7.0);
        assert_eq!(line.r2, 1.0);
    }

    #[test]
    fn fewer_than_two_points_are_no_line() {
        let one = swings(&[(4, 9.0)]);
        let line = Line::fit(one.iter().copied(), 8);
        assert_eq!(
            (line.slope, line.level, line.r2, line.points),
            (0.0, 9.0, 0.0, 1)
        );
        let none = Line::fit(std::iter::empty(), 8);
        assert_eq!((none.slope, none.level, none.points), (0.0, 0.0, 0));
        let lines = Lines {
            upper: line,
            lower: line,
        };
        assert!(!lines.fitted());
    }

    #[test]
    fn the_first_bar_can_be_a_swing_point() {
        // The window holds only the bars there are, as the oracle pads it.
        let mut highs = SwingFinder::highs(1);
        assert_eq!(highs.push(5.0), None);
        assert_eq!(highs.push(4.0), Some(5.0));
        assert_eq!(highs.push(3.0), None);
    }

    #[test]
    fn a_swing_ties_with_an_equal_neighbour() {
        let mut lows = SwingFinder::lows(1);
        let found: Vec<Option<f64>> = [3.0, 1.0, 1.0, 2.0, 0.5]
            .iter()
            .map(|&v| lows.push(v))
            .collect();
        assert_eq!(found, [None, None, Some(1.0), Some(1.0), None]);
    }

    #[test]
    fn repeats_within_pivot_n_bars_fold_into_the_first() {
        let mut log = ClusteredLog::new(5, 100);
        for at in [10, 15, 21, 22] {
            log.push(Swing { at, price: 1.0 });
        }
        let kept: Vec<usize> = log.log.iter().map(|s| s.at).collect();
        assert_eq!(kept, [10, 21]);
    }

    #[test]
    fn a_neckline_is_read_along_its_line() {
        let neckline = Neckline {
            armed_at: 12,
            left: Swing { at: 0, price: 10.0 },
            right: Swing {
                at: 10,
                price: 20.0,
            },
        };
        assert_eq!(neckline.level(5), 15.0);
        assert_eq!(neckline.level(14), 24.0);
    }

    #[test]
    fn the_window_keeps_swings_confirmed_period_bars_ago() {
        let mut log = SwingLog::new(10);
        for at in [0, 4, 9, 15] {
            log.push(Swing { at, price: 1.0 });
        }
        log.trim(14);
        let kept: Vec<usize> = log.iter().map(|s| s.at).collect();
        assert_eq!(kept, [4, 9, 15]);
    }
}
