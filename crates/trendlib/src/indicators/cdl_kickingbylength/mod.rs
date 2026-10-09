use crate::core::candles::{Candle, CandleSetting, PatternState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_kickingbylength";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 11;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = PatternState;

/// A kicking pattern answered with the colour of the longer of its two bodies
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    let prior = pattern.back(1);
    let marubozu = |candle: Candle, back: usize| {
        candle.body() > pattern.average(CandleSetting::BODY_LONG, back)
            && candle.upper_shadow() < pattern.average(CandleSetting::SHADOW_VERY_SHORT, back)
            && candle.lower_shadow() < pattern.average(CandleSetting::SHADOW_VERY_SHORT, back)
    };
    let jumped =
        (prior.is_black() && bar.gaps_above(prior)) || (prior.is_white() && bar.gaps_below(prior));
    let kicking =
        bar.is_white() != prior.is_white() && marubozu(prior, 1) && marubozu(bar, 0) && jumped;
    if kicking {
        // The longer marubozu decides, not the later one.
        if bar.body() > prior.body() {
            bar.colour() * 100.0
        } else {
            prior.colour() * 100.0
        }
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CdlKickingbylength;

pub type CdlKickingbylengthStream = BarStream<CdlKickingbylength, 4, 1>;

impl Kernel<4, 1> for CdlKickingbylength {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_kickingbylength"];

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
