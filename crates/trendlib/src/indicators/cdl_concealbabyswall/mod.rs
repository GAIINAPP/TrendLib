use crate::core::candles::{Candle, CandleSetting, PatternState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_concealbabyswall";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 13;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = PatternState;

/// Four black bars, two of them marubozu, then one that swallows the bar before it whole
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    let third = pattern.back(1);
    let second = pattern.back(2);
    let first = pattern.back(3);
    let marubozu = |candle: Candle, back: usize| {
        let very_short = pattern.average(CandleSetting::SHADOW_VERY_SHORT, back);
        candle.lower_shadow() < very_short && candle.upper_shadow() < very_short
    };
    if first.is_black()
        && second.is_black()
        && third.is_black()
        && bar.is_black()
        && marubozu(first, 3)
        && marubozu(second, 2)
        && third.body_gaps_below(second)
        && third.upper_shadow() > pattern.average(CandleSetting::SHADOW_VERY_SHORT, 1)
        && third.high > second.close
        && bar.high > third.high
        && bar.low < third.low
    {
        100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CdlConcealbabyswall;

pub type CdlConcealbabyswallStream = BarStream<CdlConcealbabyswall, 4, 1>;

impl Kernel<4, 1> for CdlConcealbabyswall {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_concealbabyswall"];

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
