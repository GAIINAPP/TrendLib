use crate::core::chart::whole;
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::{Lagged, RollingExtreme};

pub const NAME: &str = "frama";

pub const PERIOD_DEFAULT: usize = 16;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub period: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
struct Span {
    highest: RollingExtreme,
    lowest: RollingExtreme,
}

impl Span {
    fn new(period: usize) -> Self {
        Self {
            highest: RollingExtreme::highest(period),
            lowest: RollingExtreme::lowest(period),
        }
    }

    fn push(&mut self, high: f64, low: f64) -> Option<f64> {
        let top = self.highest.push(high);
        let bottom = self.lowest.push(low);
        Some(top? - bottom?)
    }
}

#[derive(Clone, Debug)]
pub struct State {
    newer: Span,
    older: Span,
    whole: Span,
    held_high: Lagged,
    held_low: Lagged,
    half: f64,
    period: f64,
    dimension: f64,
    average: Option<f64>,
}

impl State {
    fn advance(&mut self, [high, low]: [f64; 2]) -> Option<f64> {
        let newer = self.newer.push(high, low);
        let older = match (self.held_high.push(high), self.held_low.push(low)) {
            (Some(high), Some(low)) => self.older.push(high, low),
            _ => None,
        };
        let whole = self.whole.push(high, low);
        let (newer, older, whole) = (newer?, older?, whole?);
        let (n1, n2, n3) = (newer / self.half, older / self.half, whole / self.period);
        // Ehlers keeps the last dimension when a range is zero.
        if n1 > 0.0 && n2 > 0.0 && n3 > 0.0 {
            self.dimension = ((n1 + n2).ln() - n3.ln()) / std::f64::consts::LN_2;
        }
        let alpha = (-4.6 * (self.dimension - 1.0)).exp().clamp(0.01, 1.0);
        let price = (high + low) / 2.0;
        let next = match self.average {
            None => price,
            Some(previous) => previous + alpha * (price - previous),
        };
        self.average = Some(next);
        Some(next)
    }
}

impl Step<2, 1> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 1]> {
        self.advance(bar).map(|value| [value])
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 1]> {
        self.clone().advance(bar).map(|value| [value])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Frama;

pub type FramaStream = BarStream<Frama, 2, 1>;

impl Kernel<2, 1> for Frama {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["high", "low"];
    const OUTPUTS: [&'static str; 1] = ["frama"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(NAME, "period", params.period, PERIOD_MIN, PERIOD_MAX)?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.period - 1
    }

    fn state(params: &Params) -> State {
        let half = params.period / 2;
        State {
            newer: Span::new(half),
            older: Span::new(half),
            whole: Span::new(params.period),
            held_high: Lagged::new(half),
            held_low: Lagged::new(half),
            half: half as f64,
            period: params.period as f64,
            dimension: 0.0,
            average: None,
        }
    }
}
