use crate::core::candles::{CandleSetting, PatternState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_mathold";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 14;

pub const PENETRATION_DEFAULT: f64 = 0.5;
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

/// A long white bar, three small ones drifting back without undoing it, then another long white one
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    let fourth = pattern.back(1);
    let third = pattern.back(2);
    let second = pattern.back(3);
    let first = pattern.back(4);
    // How far down the first body the three resting bars may reach.
    let floor = first.close - first.body() * pattern.penetration;
    if first.is_white()
        && second.is_black()
        && bar.is_white()
        && first.body() > pattern.average(CandleSetting::BODY_LONG, 4)
        && second.body() < pattern.average(CandleSetting::BODY_SHORT, 3)
        && third.body() < pattern.average(CandleSetting::BODY_SHORT, 2)
        && fourth.body() < pattern.average(CandleSetting::BODY_SHORT, 1)
        && second.body_gaps_above(first)
        && third.body_bottom() < first.close
        && fourth.body_bottom() < first.close
        && third.body_bottom() > floor
        && fourth.body_bottom() > floor
        && third.body_top() < second.open
        && fourth.body_top() < third.body_top()
        && bar.open > fourth.close
        && bar.close > second.high.max(third.high).max(fourth.high)
    {
        100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CdlMathold;

pub type CdlMatholdStream = BarStream<CdlMathold, 4, 1>;

impl Kernel<4, 1> for CdlMathold {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_mathold"];

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
