use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::RollingWindow;

pub const NAME: &str = "ultosc";

pub const PERIOD1_DEFAULT: usize = 7;
pub const PERIOD2_DEFAULT: usize = 14;
pub const PERIOD3_DEFAULT: usize = 28;
pub const PERIOD_MIN: usize = 1;
pub const PERIOD_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub period1: usize,
    pub period2: usize,
    pub period3: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period1: PERIOD1_DEFAULT,
            period2: PERIOD2_DEFAULT,
            period3: PERIOD3_DEFAULT,
        }
    }
}

/// One window's share of its true range that the close claimed.
#[derive(Clone, Debug)]
struct Pressure {
    bought: RollingWindow,
    range: RollingWindow,
}

impl Pressure {
    fn new(period: usize) -> Self {
        Self {
            bought: RollingWindow::new(period),
            range: RollingWindow::new(period),
        }
    }

    fn push(&mut self, bought: f64, range: f64) -> Option<f64> {
        let full = self.bought.push(bought);
        self.range.push(range);
        full.then(|| ratio(self.bought.sum(), self.range.sum()))
    }

    fn preview(&self, bought: f64, range: f64) -> Option<f64> {
        self.bought.preview_is_full().then(|| {
            ratio(
                self.bought.preview_sum(bought),
                self.range.preview_sum(range),
            )
        })
    }
}

/// A window with no true range at all has no share to report, and TA-Lib
/// answers zero rather than dividing. The test is exact.
fn ratio(bought: f64, range: f64) -> f64 {
    if range == 0.0 { 0.0 } else { bought / range }
}

#[derive(Clone, Debug)]
pub struct State {
    previous_close: Option<f64>,
    short: Pressure,
    middle: Pressure,
    long: Pressure,
}

/// The bar's close measured against the range it shared with the bar before
/// it, which is what makes a gap count as part of the move.
fn bar(high: f64, low: f64, close: f64, previous_close: f64) -> (f64, f64) {
    let floor = low.min(previous_close);
    let ceiling = high.max(previous_close);
    (close - floor, ceiling - floor)
}

fn blend(short: f64, middle: f64, long: f64) -> f64 {
    100.0 * (4.0 * short + 2.0 * middle + long) / 7.0
}

impl Step<3, 1> for State {
    fn push(&mut self, input: [f64; 3]) -> Option<[f64; 1]> {
        let previous = self.previous_close.replace(input[2])?;
        let (bought, range) = bar(input[0], input[1], input[2], previous);
        // Every window has to take the bar before any of them can bail, or the
        // longer ones would miss what the shorter ones consumed.
        let short = self.short.push(bought, range);
        let middle = self.middle.push(bought, range);
        let long = self.long.push(bought, range)?;
        Some([blend(short?, middle?, long)])
    }

    fn preview(&self, input: [f64; 3]) -> Option<[f64; 1]> {
        let previous = self.previous_close?;
        let (bought, range) = bar(input[0], input[1], input[2], previous);
        Some([blend(
            self.short.preview(bought, range)?,
            self.middle.preview(bought, range)?,
            self.long.preview(bought, range)?,
        )])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Ultosc;

pub type UltoscStream = BarStream<Ultosc, 3, 1>;

impl Kernel<3, 1> for Ultosc {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["ultosc"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        for (name, value) in [
            ("period1", params.period1),
            ("period2", params.period2),
            ("period3", params.period3),
        ] {
            if !(PERIOD_MIN..=PERIOD_MAX).contains(&value) {
                return Err(TlError::param_out_of_range(
                    NAME, name, value, PERIOD_MIN, PERIOD_MAX,
                ));
            }
        }
        Ok(())
    }

    // The longest window decides, plus the bar each range is measured against.
    fn lookback(params: &Params) -> usize {
        params.period1.max(params.period2).max(params.period3)
    }

    fn state(params: &Params) -> State {
        State {
            previous_close: None,
            short: Pressure::new(params.period1),
            middle: Pressure::new(params.period2),
            long: Pressure::new(params.period3),
        }
    }
}
