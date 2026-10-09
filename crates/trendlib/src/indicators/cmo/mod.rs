use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::WilderSum;

pub const NAME: &str = "cmo";

pub const PERIOD_DEFAULT: usize = 14;
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
pub struct State {
    previous: Option<f64>,
    gains: WilderSum,
    losses: WilderSum,
}

impl State {
    fn split(change: f64) -> (f64, f64) {
        if change < 0.0 {
            (0.0, -change)
        } else {
            (change, 0.0)
        }
    }

    // A stretch with no movement at all leaves both totals at zero. The
    // oscillator is reported as 0 there, as TA-Lib reports it; the ratio is
    // undefined.
    fn oscillator(gains: f64, losses: f64) -> f64 {
        let total = gains + losses;
        if total == 0.0 {
            0.0
        } else {
            100.0 * (gains - losses) / total
        }
    }
}

impl Step<1, 1> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 1]> {
        let previous = self.previous.replace(bar[0])?;
        let (gain, loss) = Self::split(bar[0] - previous);
        match (self.gains.push(gain), self.losses.push(loss)) {
            (Some(gains), Some(losses)) => Some([Self::oscillator(gains, losses)]),
            _ => None,
        }
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        let previous = self.previous?;
        let (gain, loss) = Self::split(bar[0] - previous);
        match (self.gains.preview(gain), self.losses.preview(loss)) {
            (Some(gains), Some(losses)) => Some([Self::oscillator(gains, losses)]),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cmo;

pub type CmoStream = BarStream<Cmo, 1, 1>;

impl Kernel<1, 1> for Cmo {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 1] = ["cmo"];

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

    // One bar is consumed before the first change exists, and the totals then
    // need `period` of them.
    fn lookback(params: &Params) -> usize {
        params.period
    }

    fn state(params: &Params) -> State {
        State {
            previous: None,
            gains: WilderSum::new(params.period),
            losses: WilderSum::new(params.period),
        }
    }
}
