use crate::core::chart::{at_least, whole};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::{RollingExtreme, RollingWindow};

pub const NAME: &str = "safezone";

pub const PERIOD_DEFAULT: usize = 10;
pub const PERIOD_MIN: usize = 1;
pub const PERIOD_MAX: usize = 100_000;
pub const COEFFICIENT_DEFAULT: f64 = 2.0;
pub const COEFFICIENT_MIN: f64 = 0.0;
pub const HOLD_DEFAULT: usize = 3;
pub const HOLD_MIN: usize = 1;
pub const HOLD_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub period: usize,
    pub coefficient: f64,
    pub hold: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
            coefficient: COEFFICIENT_DEFAULT,
            hold: HOLD_DEFAULT,
        }
    }
}

/// The mean of the penetrations that happened in the window, 0 when none did.
fn mean_penetration(window: &RollingWindow) -> f64 {
    let mut total = 0.0;
    let mut count = 0usize;
    for value in window.values().filter(|value| *value > 0.0) {
        total += value;
        count += 1;
    }
    if count == 0 {
        0.0
    } else {
        total / count as f64
    }
}

#[derive(Clone, Debug)]
pub struct State {
    previous: Option<[f64; 2]>,
    down: RollingWindow,
    up: RollingWindow,
    /// The levels the bar just pushed sets for the bar after it.
    pending: Option<[f64; 2]>,
    lower: RollingExtreme,
    upper: RollingExtreme,
    coefficient: f64,
}

impl State {
    fn advance(&mut self, [high, low]: [f64; 2]) -> Option<[f64; 2]> {
        let raw = self.pending.take();
        if let Some([was_high, was_low]) = self.previous {
            let below = self.down.push((was_low - low).max(0.0));
            let above = self.up.push((high - was_high).max(0.0));
            if below && above {
                self.pending = Some([
                    low - self.coefficient * mean_penetration(&self.down),
                    high + self.coefficient * mean_penetration(&self.up),
                ]);
            }
        }
        self.previous = Some([high, low]);
        let [raw_lower, raw_upper] = raw?;
        let lower = self.lower.push(raw_lower);
        let upper = self.upper.push(raw_upper);
        Some([lower?, upper?])
    }
}

impl Step<2, 2> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 2]> {
        self.advance(bar)
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 2]> {
        self.clone().advance(bar)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Safezone;

pub type SafezoneStream = BarStream<Safezone, 2, 2>;

impl Kernel<2, 2> for Safezone {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["high", "low"];
    const OUTPUTS: [&'static str; 2] = ["safezone_lower", "safezone_upper"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(NAME, "period", params.period, PERIOD_MIN, PERIOD_MAX)?;
        at_least(NAME, "coefficient", params.coefficient, COEFFICIENT_MIN)?;
        whole(NAME, "hold", params.hold, HOLD_MIN, HOLD_MAX)?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.period + params.hold
    }

    fn state(params: &Params) -> State {
        State {
            previous: None,
            down: RollingWindow::new(params.period),
            up: RollingWindow::new(params.period),
            pending: None,
            lower: RollingExtreme::highest(params.hold),
            upper: RollingExtreme::lowest(params.hold),
            coefficient: params.coefficient,
        }
    }
}
