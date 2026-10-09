use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::{TrueRange, Wilder};

pub const NAME: &str = "supertrend";

pub const PERIOD_DEFAULT: usize = 10;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100000;
pub const MULTIPLIER_DEFAULT: f64 = 3.0;
pub const MULTIPLIER_MIN: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub period: usize,
    pub multiplier: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
            multiplier: MULTIPLIER_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    range: TrueRange,
    average: Wilder,
    multiplier: f64,
    /// The bands as carried forward, and which one is being followed.
    carried: Option<Carried>,
}

#[derive(Clone, Copy, Debug)]
struct Carried {
    upper: f64,
    lower: f64,
    close: f64,
    rising: bool,
}

fn bands(bar: [f64; 3], width: f64) -> (f64, f64) {
    let middle = (bar[0] + bar[1]) / 2.0;
    (middle + width, middle - width)
}

/// A band only moves towards price, never away from it, unless the close has
/// already passed it. That is what makes the line a trailing stop.
fn carry(raw: f64, previous: f64, previous_close: f64, upper: bool) -> f64 {
    let moved_in = if upper {
        raw < previous
    } else {
        raw > previous
    };
    let broken = if upper {
        previous_close > previous
    } else {
        previous_close < previous
    };
    if moved_in || broken { raw } else { previous }
}

impl State {
    fn step(&self, bar: [f64; 3], width: f64) -> (Carried, [f64; 2]) {
        let (raw_upper, raw_lower) = bands(bar, width);
        let next = match self.carried {
            None => Carried {
                upper: raw_upper,
                lower: raw_lower,
                close: bar[2],
                // The first row has no trend to carry, so it starts rising.
                rising: true,
            },
            Some(previous) => {
                let upper = carry(raw_upper, previous.upper, previous.close, true);
                let lower = carry(raw_lower, previous.lower, previous.close, false);
                let rising = if previous.rising {
                    bar[2] >= lower
                } else {
                    bar[2] > upper
                };
                Carried {
                    upper,
                    lower,
                    close: bar[2],
                    rising,
                }
            }
        };
        let line = if next.rising { next.lower } else { next.upper };
        (next, [line, if next.rising { 1.0 } else { -1.0 }])
    }
}

impl Step<3, 2> for State {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 2]> {
        let range = self.range.push(bar[0], bar[1], bar[2]);
        let width = range.and_then(|range| self.average.push(range))? * self.multiplier;
        let (next, out) = self.step(bar, width);
        self.carried = Some(next);
        Some(out)
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 2]> {
        let range = self.range.preview(bar[0], bar[1])?;
        let width = self.average.preview(range)? * self.multiplier;
        Some(self.step(bar, width).1)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Supertrend;

pub type SupertrendStream = BarStream<Supertrend, 3, 2>;

impl Kernel<3, 2> for Supertrend {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 2] = ["supertrend", "supertrend_direction"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        if !(PERIOD_MIN..=PERIOD_MAX).contains(&params.period) {
            return Err(TlError::param_out_of_range(
                NAME,
                "period",
                params.period,
                PERIOD_MIN,
                PERIOD_MAX,
            ));
        }
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.period
    }

    fn state(params: &Params) -> State {
        State {
            range: TrueRange::new(),
            average: Wilder::new(params.period),
            multiplier: params.multiplier,
            carried: None,
        }
    }
}
