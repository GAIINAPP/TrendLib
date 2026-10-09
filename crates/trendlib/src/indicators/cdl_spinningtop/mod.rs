use crate::core::candles::{CandleSetting, PatternState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_spinningtop";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 10;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = PatternState;

/// A short body with a wick longer than itself on each side
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    if bar.body() < pattern.average(CandleSetting::BODY_SHORT, 0)
        && bar.upper_shadow() > bar.body()
        && bar.lower_shadow() > bar.body()
    {
        bar.colour() * 100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CdlSpinningtop;

pub type CdlSpinningtopStream = BarStream<CdlSpinningtop, 4, 1>;

impl Kernel<4, 1> for CdlSpinningtop {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_spinningtop"];

    type Params = Params;
    type State = State;

    fn validate(_params: &Params) -> Result<(), TlError> {
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        LOOKBACK
    }

    fn state(params: &Params) -> State {
        let _ = params;
        PatternState::new(LOOKBACK, 0.0, detect)
    }
}
