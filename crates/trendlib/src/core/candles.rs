//! What the candlestick patterns are written in terms of.
//!
//! A pattern asks questions like "is this body long?", and long is not a fixed
//! size: it is measured against how long the bodies around it have been. The
//! settings below are TA-Lib's defaults, which say for each question which
//! part of a bar to measure, how many bars to average it over and what to
//! multiply that average by. 0.1 ships them fixed; making them configurable is
//! M6.

/// Which part of a bar a setting measures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RangeType {
    /// The body: how far the close is from the open.
    RealBody,
    /// The whole bar, wicks included.
    HighLow,
    /// The two wicks together, which is why their average is halved.
    Shadows,
}

/// One bar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

impl Candle {
    pub fn new(bar: [f64; 4]) -> Self {
        Self {
            open: bar[0],
            high: bar[1],
            low: bar[2],
            close: bar[3],
        }
    }

    pub fn body(self) -> f64 {
        (self.close - self.open).abs()
    }

    pub fn range(self) -> f64 {
        self.high - self.low
    }

    pub fn body_top(self) -> f64 {
        self.open.max(self.close)
    }

    pub fn body_bottom(self) -> f64 {
        self.open.min(self.close)
    }

    pub fn upper_shadow(self) -> f64 {
        self.high - self.body_top()
    }

    pub fn lower_shadow(self) -> f64 {
        self.body_bottom() - self.low
    }

    pub fn shadows(self) -> f64 {
        self.upper_shadow() + self.lower_shadow()
    }

    /// `true` for a bar that closed at or above its open. TA-Lib counts a bar
    /// that opened and closed at the same price as white, so the test is not
    /// strict.
    pub fn is_white(self) -> bool {
        self.close >= self.open
    }

    pub fn is_black(self) -> bool {
        !self.is_white()
    }

    /// +1 for white and -1 for black, which several patterns multiply by.
    pub fn colour(self) -> f64 {
        if self.is_white() { 1.0 } else { -1.0 }
    }

    /// The body sits entirely above the earlier one's, wicks ignored.
    pub fn body_gaps_above(self, earlier: Candle) -> bool {
        self.body_bottom() > earlier.body_top()
    }

    /// The body sits entirely below the earlier one's, wicks ignored.
    pub fn body_gaps_below(self, earlier: Candle) -> bool {
        self.body_top() < earlier.body_bottom()
    }

    /// The whole bar sits above the earlier one, wicks included.
    pub fn gaps_above(self, earlier: Candle) -> bool {
        self.low > earlier.high
    }

    /// The whole bar sits below the earlier one, wicks included.
    pub fn gaps_below(self, earlier: Candle) -> bool {
        self.high < earlier.low
    }

    fn measured(self, range: RangeType) -> f64 {
        match range {
            RangeType::RealBody => self.body(),
            RangeType::HighLow => self.range(),
            RangeType::Shadows => self.shadows(),
        }
    }
}

/// One of TA-Lib's candle settings: what to measure, over how many bars, and
/// what to multiply the average by.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CandleSetting {
    pub range: RangeType,
    pub avg_period: usize,
    pub factor: f64,
}

impl CandleSetting {
    pub const BODY_LONG: Self = Self::of(RangeType::RealBody, 10, 1.0);
    pub const BODY_VERY_LONG: Self = Self::of(RangeType::RealBody, 10, 3.0);
    pub const BODY_SHORT: Self = Self::of(RangeType::RealBody, 10, 1.0);
    pub const BODY_DOJI: Self = Self::of(RangeType::HighLow, 10, 0.1);
    pub const SHADOW_LONG: Self = Self::of(RangeType::RealBody, 0, 1.0);
    pub const SHADOW_VERY_LONG: Self = Self::of(RangeType::RealBody, 0, 2.0);
    pub const SHADOW_SHORT: Self = Self::of(RangeType::Shadows, 10, 1.0);
    pub const SHADOW_VERY_SHORT: Self = Self::of(RangeType::HighLow, 10, 0.1);
    pub const NEAR: Self = Self::of(RangeType::HighLow, 5, 0.2);
    pub const FAR: Self = Self::of(RangeType::HighLow, 5, 0.6);
    pub const EQUAL: Self = Self::of(RangeType::HighLow, 5, 0.05);

    const fn of(range: RangeType, avg_period: usize, factor: f64) -> Self {
        Self {
            range,
            avg_period,
            factor,
        }
    }
}

/// The bars a pattern reads, newest last.
///
/// `back(0)` is the bar being judged and `back(1)` the one before it, which is
/// how the patterns are written. `average` measures a setting over the bars
/// **before** the one it is asked about, so a bar is never measured against
/// itself.
#[derive(Clone, Debug)]
pub struct CandleHistory {
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

impl CandleHistory {
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

    pub fn is_full(&self) -> bool {
        self.seen >= self.bars.len()
    }

    /// The bar `back` places before the newest one pushed.
    pub fn back(&self, back: usize) -> Candle {
        let len = self.bars.len();
        self.bars[(self.head + len - 1 - back) % len]
    }

    /// The threshold `setting` gives for the bar `back` places ago.
    ///
    /// The sum is re-added from the window rather than carried forward, so the
    /// comparison a pattern makes does not depend on how many bars came before
    /// the ones it is looking at.
    pub fn average(&self, setting: CandleSetting, back: usize) -> f64 {
        let measured = if setting.avg_period == 0 {
            // No period means the bar is measured against itself.
            self.back(back).measured(setting.range)
        } else {
            let total: f64 = (1..=setting.avg_period)
                .map(|step| self.back(back + step).measured(setting.range))
                .sum();
            total / setting.avg_period as f64
        };
        // Two shadows were added together, so their average is per shadow.
        let halve = if setting.range == RangeType::Shadows {
            2.0
        } else {
            1.0
        };
        measured * setting.factor / halve
    }
}

/// The scaffolding every pattern shares.
///
/// A pattern differs from its neighbours only in how far back it reads and
/// what it asks of those bars, so that is all its folder holds: a lookback
/// taken from the oracle and one function that answers 100, -100 or 0.
#[derive(Clone, Debug)]
pub struct PatternState {
    history: CandleHistory,
    /// How far a close must travel into the previous body for the patterns
    /// that ask; zero for the ones that do not.
    pub penetration: f64,
    detect: fn(&PatternState) -> f64,
}

impl PatternState {
    pub fn new(lookback: usize, penetration: f64, detect: fn(&PatternState) -> f64) -> Self {
        Self {
            history: CandleHistory::new(lookback),
            penetration,
            detect,
        }
    }

    /// The bar `back` places before the newest one.
    pub fn back(&self, back: usize) -> Candle {
        self.history.back(back)
    }

    /// The threshold `setting` gives for the bar `back` places ago.
    pub fn average(&self, setting: CandleSetting, back: usize) -> f64 {
        self.history.average(setting, back)
    }
}

impl crate::core::kernel::Step<4, 1> for PatternState {
    fn push(&mut self, bar: [f64; 4]) -> Option<[f64; 1]> {
        self.history.push(Candle::new(bar));
        self.history.is_full().then(|| [(self.detect)(self)])
    }

    fn preview(&self, bar: [f64; 4]) -> Option<[f64; 1]> {
        let mut forked = self.clone();
        forked.history.push(Candle::new(bar));
        forked.history.is_full().then(|| [(forked.detect)(&forked)])
    }
}
