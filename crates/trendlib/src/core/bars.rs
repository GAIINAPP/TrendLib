//! What the bar patterns are written in terms of.
//!
//! A bar pattern asks a fixed question of
//! the last few bars: did this one open beyond the two before it, is its range
//! the narrowest of seven. Each folder holds the question; the bars, the
//! warm-up and the stream are here, shared.

use crate::core::candles::Candle;

/// The most recent bars, newest last.
#[derive(Clone, Debug)]
pub struct BarHistory {
    bars: Box<[Candle]>,
    head: usize,
    seen: usize,
}

const EMPTY: Candle = Candle {
    open: 0.0,
    high: 0.0,
    low: 0.0,
    close: 0.0,
};

impl BarHistory {
    /// Room for the current bar and `depth` before it.
    pub fn new(depth: usize) -> Self {
        Self {
            bars: vec![EMPTY; depth + 1].into_boxed_slice(),
            head: 0,
            seen: 0,
        }
    }

    pub fn push(&mut self, bar: Candle) {
        self.bars[self.head] = bar;
        self.head = (self.head + 1) % self.bars.len();
        self.seen += 1;
    }

    /// Bars pushed so far, counted from the first valid one.
    pub fn seen(&self) -> usize {
        self.seen
    }

    /// The bar `back` places before the newest; `back(0)` is the newest.
    pub fn back(&self, back: usize) -> Candle {
        let len = self.bars.len();
        self.bars[(self.head + len - 1 - back) % len]
    }

    /// Whether the bar `back` places before the newest exists.
    pub fn has(&self, back: usize) -> bool {
        back < self.seen
    }
}

/// A bar pattern's question: `+100`, `-100` or `0` for the newest bar.
pub type BarRule<P> = fn(&BarHistory, &P) -> f64;

/// A bar pattern: its parameters, how many bars it keeps and reads, and its
/// question. It answers from the bar `lookback` on, the first the oracle reads.
#[derive(Clone, Debug)]
pub struct BarState<P: Copy> {
    history: BarHistory,
    params: P,
    lookback: usize,
    rule: BarRule<P>,
}

impl<P: Copy> BarState<P> {
    /// `depth` is how many bars before the current one the rule reads, at
    /// least `lookback`.
    pub fn new(depth: usize, lookback: usize, params: P, rule: BarRule<P>) -> Self {
        Self {
            history: BarHistory::new(depth.max(lookback)),
            params,
            lookback,
            rule,
        }
    }
}

impl<P: Copy + std::fmt::Debug> crate::core::kernel::Step<4, 1> for BarState<P> {
    fn push(&mut self, bar: [f64; 4]) -> Option<[f64; 1]> {
        self.history.push(Candle::new(bar));
        (self.history.seen() > self.lookback).then(|| [(self.rule)(&self.history, &self.params)])
    }

    fn preview(&self, bar: [f64; 4]) -> Option<[f64; 1]> {
        let mut forked = self.clone();
        forked.push(bar)
    }
}

/// One reading from a pattern's two halves, the way the oracle combines them:
/// each half adds its sign and the sum is clipped, so both firing at once
/// reads 0.
pub fn combine(rising: bool, falling: bool) -> f64 {
    match (rising, falling) {
        (true, false) => 100.0,
        (false, true) => -100.0,
        _ => 0.0,
    }
}

/// The highest high of the `count` bars from `from` back to `from + count - 1`
/// places before the newest.
pub fn highest_high(history: &BarHistory, from: usize, count: usize) -> f64 {
    (from..from + count)
        .map(|back| history.back(back).high)
        .fold(f64::NEG_INFINITY, f64::max)
}

/// The lowest low of the same bars.
pub fn lowest_low(history: &BarHistory, from: usize, count: usize) -> f64 {
    (from..from + count)
        .map(|back| history.back(back).low)
        .fold(f64::INFINITY, f64::min)
}
