use crate::core::candles::{CandleSetting, PatternState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_eveningstar";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 12;

pub const PENETRATION_DEFAULT: f64 = 0.3;
pub const PENETRATION_MIN: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub penetration: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            penetration: PENETRATION_DEFAULT,
        }
    }
}

pub type State = PatternState;

/// A long white bar, a short one gapping up from it, then a black one closing well back into it
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    let middle = pattern.back(1);
    let first = pattern.back(2);
    let reached = first.close - first.body() * pattern.penetration;
    if first.body() > pattern.average(CandleSetting::BODY_LONG, 2)
        && first.is_white()
        && middle.body() <= pattern.average(CandleSetting::BODY_SHORT, 1)
        && middle.body_gaps_above(first)
        && bar.body() > pattern.average(CandleSetting::BODY_SHORT, 0)
        && bar.is_black()
        && bar.close < reached
    {
        -100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CdlEveningstar;

pub type CdlEveningstarStream = BarStream<CdlEveningstar, 4, 1>;

impl Kernel<4, 1> for CdlEveningstar {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_eveningstar"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        if !params.penetration.is_finite() || params.penetration < PENETRATION_MIN {
            return Err(TlError::float_param_out_of_range(
                NAME,
                "penetration",
                params.penetration,
                PENETRATION_MIN,
                f64::INFINITY,
            ));
        }
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        LOOKBACK
    }

    fn state(params: &Params) -> State {
        let _ = params;
        PatternState::new(LOOKBACK, params.penetration, detect)
    }
}
