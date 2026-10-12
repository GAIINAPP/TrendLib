use crate::core::chart::{SwingFinder, whole};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "fib_retracement";

pub const PIVOT_N_DEFAULT: usize = 5;
pub const PIVOT_N_MIN: usize = 1;
pub const PIVOT_N_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub pivot_n: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            pivot_n: PIVOT_N_DEFAULT,
        }
    }
}

/// The share of a leg each level between its two ends has given back.
const RATIOS: [f64; 5] = [0.236, 0.382, 0.5, 0.618, 0.786];

/// The levels of a leg that ran from `start` to `end`: its end, each ratio of
/// the way back to the start, and its start.
///
/// The two ends are the swing prices themselves rather than `end + 0 * span`
/// and `end + 1 * span`, which need not give `start` back exactly.
pub(crate) fn levels(start: f64, end: f64) -> [f64; 7] {
    let span = start - end;
    [
        end,
        end + RATIOS[0] * span,
        end + RATIOS[1] * span,
        end + RATIOS[2] * span,
        end + RATIOS[3] * span,
        end + RATIOS[4] * span,
        start,
    ]
}

/// The last leg of a zigzag through the confirmed swing points.
#[derive(Clone, Copy, Debug)]
struct Leg {
    /// The turning point the leg ends at, and whether it is a high.
    end: Option<(bool, f64)>,
    start: Option<f64>,
}

impl Leg {
    /// Files one swing point. Further out on the side the leg ends on, it
    /// moves the end; on the other side and beyond the end, it starts the next
    /// leg. Anything else is passed over: a lower high after the top is part
    /// of the pullback, and a high under the last low has no leg to end.
    fn file(&mut self, is_high: bool, price: f64) {
        if let Some((side, last)) = self.end {
            let beyond = if is_high { price > last } else { price < last };
            if !beyond {
                return;
            }
            if side != is_high {
                self.start = Some(last);
            }
        }
        self.end = Some((is_high, price));
    }

    /// The leg once a bar has confirmed these swing points. The low is filed
    /// first, so a bar that confirms both ends the leg at its high.
    fn after(mut self, high: Option<f64>, low: Option<f64>) -> Self {
        if let Some(price) = low {
            self.file(false, price);
        }
        if let Some(price) = high {
            self.file(true, price);
        }
        self
    }

    fn levels(self) -> [f64; 7] {
        match (self.start, self.end) {
            (Some(start), Some((_, end))) => levels(start, end),
            _ => [f64::NAN; 7],
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    highs: SwingFinder,
    lows: SwingFinder,
    leg: Leg,
    lookback: usize,
    bars: usize,
}

impl Step<2, 7> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 7]> {
        let high = self.highs.push(bar[0]);
        let low = self.lows.push(bar[1]);
        self.leg = self.leg.after(high, low);
        let now = self.bars;
        self.bars += 1;
        (now >= self.lookback).then(|| self.leg.levels())
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 7]> {
        let leg = self
            .leg
            .after(self.highs.preview(bar[0]), self.lows.preview(bar[1]));
        (self.bars >= self.lookback).then(|| leg.levels())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FibRetracement;

pub type FibRetracementStream = BarStream<FibRetracement, 2, 7>;

impl Kernel<2, 7> for FibRetracement {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["high", "low"];
    const OUTPUTS: [&'static str; 7] = [
        "fib_0", "fib_236", "fib_382", "fib_500", "fib_618", "fib_786", "fib_1000",
    ];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(NAME, "pivot_n", params.pivot_n, PIVOT_N_MIN, PIVOT_N_MAX)
    }

    /// The first bar that can confirm a swing point with `pivot_n` bars on
    /// each side of it. Swing points nearer the start are confirmed sooner and
    /// are held; an input too short to reach this bar is one the oracle finds
    /// no swing point in at all.
    fn lookback(params: &Params) -> usize {
        2 * params.pivot_n
    }

    fn state(params: &Params) -> State {
        State {
            highs: SwingFinder::highs(params.pivot_n),
            lows: SwingFinder::lows(params.pivot_n),
            leg: Leg {
                end: None,
                start: None,
            },
            lookback: Self::lookback(params),
            bars: 0,
        }
    }
}
