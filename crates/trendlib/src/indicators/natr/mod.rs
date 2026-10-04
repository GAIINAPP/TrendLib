use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::{TrueRange, Wilder};

pub const NAME: &str = "natr";

pub const PERIOD_DEFAULT: usize = 14;
pub const PERIOD_MIN: usize = 1;
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
pub struct State {
    range: TrueRange,
    average: Wilder,
}

// Dividing by the close is what makes this comparable across instruments, and
// it is also the one way the value can fail to exist: a close of zero leaves it
// undefined rather than infinite.
fn normalise(average: f64, close: f64) -> f64 {
    if close == 0.0 {
        f64::NAN
    } else {
        100.0 * average / close
    }
}

impl Step<3, 1> for State {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 1]> {
        let range = self.range.push(bar[0], bar[1], bar[2])?;
        self.average
            .push(range)
            .map(|average| [normalise(average, bar[2])])
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 1]> {
        let range = self.range.preview(bar[0], bar[1])?;
        self.average
            .preview(range)
            .map(|average| [normalise(average, bar[2])])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Natr;

pub type NatrStream = BarStream<Natr, 3, 1>;

impl Kernel<3, 1> for Natr {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["natr"];

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
        }
    }
}
