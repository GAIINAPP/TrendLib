use crate::core::bars::{BarHistory, BarState};
use crate::core::chart::EPS;
use crate::core::chart::at_least;
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "bar_pipe_bottom";

pub const HEIGHT_TOL_DEFAULT: f64 = 0.02;
pub const HEIGHT_TOL_MIN: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub height_tol: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            height_tol: HEIGHT_TOL_DEFAULT,
        }
    }
}

pub type State = BarState<Params>;

/// The oracle's 14-bar average true range ending at this bar. The first bar's
/// true range uses its own close as the previous one, as the oracle's does.
fn average_true_range(history: &BarHistory) -> f64 {
    let total: f64 = (0..14)
        .map(|back| {
            let bar = history.back(back);
            let previous = if history.has(back + 1) {
                history.back(back + 1).close
            } else {
                bar.close
            };
            (bar.high - bar.low).max((bar.high - previous).abs().max((bar.low - previous).abs()))
        })
        .sum();
    total / 14.0
}

/// Two tall bars side by side whose lows match.
fn rule(history: &BarHistory, params: &Params) -> f64 {
    let (bar, previous) = (history.back(0), history.back(1));
    let atr = average_true_range(history);
    let tall = |range: f64| range > atr * 1.2;
    if atr > 0.0
        && tall(previous.high - previous.low)
        && tall(bar.high - bar.low)
        && (bar.low - previous.low).abs() / (bar.low.abs() + EPS) < params.height_tol
        && bar.close > bar.open
    {
        100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BarPipeBottom;

pub type BarPipeBottomStream = BarStream<BarPipeBottom, 4, 1>;

impl Kernel<4, 1> for BarPipeBottom {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["bar_pipe_bottom"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        at_least(NAME, "height_tol", params.height_tol, HEIGHT_TOL_MIN)?;
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        13
    }

    fn state(params: &Params) -> State {
        BarState::new(14, 13, *params, rule)
    }
}
