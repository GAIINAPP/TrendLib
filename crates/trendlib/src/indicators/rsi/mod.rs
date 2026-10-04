use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::Wilder;

pub const NAME: &str = "rsi";

// Mirrors spec.yaml. From M2 `cargo xtask generate` owns these three values and
// the validate body below; a Python test asserts they agree until then.
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
    gain: Wilder,
    loss: Wilder,
}

impl State {
    fn split(change: f64) -> (f64, f64) {
        if change < 0.0 {
            (0.0, -change)
        } else {
            (change, 0.0)
        }
    }

    // A flat stretch leaves both averages at zero. TA-Lib reports 0 there, and
    // so does TrendLib; any other choice would divide by zero.
    fn index(gain: f64, loss: f64) -> f64 {
        let total = gain + loss;
        if total == 0.0 {
            0.0
        } else {
            100.0 * gain / total
        }
    }
}

impl Step<1, 1> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 1]> {
        let previous = self.previous.replace(bar[0])?;
        let (gain, loss) = Self::split(bar[0] - previous);
        match (self.gain.push(gain), self.loss.push(loss)) {
            (Some(gain), Some(loss)) => Some([Self::index(gain, loss)]),
            _ => None,
        }
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        let previous = self.previous?;
        let (gain, loss) = Self::split(bar[0] - previous);
        match (self.gain.preview(gain), self.loss.preview(loss)) {
            (Some(gain), Some(loss)) => Some([Self::index(gain, loss)]),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rsi;

pub type RsiStream = BarStream<Rsi, 1, 1>;

impl Kernel<1, 1> for Rsi {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 1] = ["rsi"];

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
            previous: None,
            gain: Wilder::new(params.period),
            loss: Wilder::new(params.period),
        }
    }
}
