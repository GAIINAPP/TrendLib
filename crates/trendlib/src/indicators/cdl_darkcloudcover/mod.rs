use crate::core::candles::{CandleSetting, PatternState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_darkcloudcover";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 11;

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

/// A black bar that opened above the previous high and closed well into the white body before it
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    let prior = pattern.back(1);
    if prior.is_white()
        && prior.body() > pattern.average(CandleSetting::BODY_LONG, 1)
        && bar.is_black()
        && bar.open > prior.high
        && bar.close > prior.open
        && bar.close < prior.close - prior.body() * pattern.penetration
    {
        -100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CdlDarkcloudcover;

pub type CdlDarkcloudcoverStream = BarStream<CdlDarkcloudcover, 4, 1>;

impl Kernel<4, 1> for CdlDarkcloudcover {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_darkcloudcover"];

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
