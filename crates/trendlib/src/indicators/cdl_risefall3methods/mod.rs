use crate::core::candles::{Candle, CandleSetting, PatternState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_risefall3methods";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 14;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = PatternState;

/// A long bar, three short ones drifting back inside its range, then a long one carrying on
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    let fourth = pattern.back(1);
    let third = pattern.back(2);
    let second = pattern.back(3);
    let first = pattern.back(4);
    // Part of the body has to fall inside the first bar's range, not all of
    // it: a middle bar may run past either end so long as it still overlaps.
    let within =
        |candle: Candle| candle.body_bottom() < first.high && candle.body_top() > first.low;
    // The middle bars move against the first, and the last one takes it all
    // back; multiplying by the first bar's colour reads both directions at
    // once, which is how TA-Lib writes it.
    let way = first.colour();
    if first.body() > pattern.average(CandleSetting::BODY_LONG, 4)
        && second.body() < pattern.average(CandleSetting::BODY_SHORT, 3)
        && third.body() < pattern.average(CandleSetting::BODY_SHORT, 2)
        && fourth.body() < pattern.average(CandleSetting::BODY_SHORT, 1)
        && bar.body() > pattern.average(CandleSetting::BODY_LONG, 0)
        && first.is_white() != second.is_white()
        && second.is_white() == third.is_white()
        && third.is_white() == fourth.is_white()
        && fourth.is_white() != bar.is_white()
        && within(second)
        && within(third)
        && within(fourth)
        && third.close * way < second.close * way
        && fourth.close * way < third.close * way
        && bar.open * way > fourth.close * way
        && bar.close * way > first.close * way
    {
        way * 100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CdlRisefall3methods;

pub type CdlRisefall3methodsStream = BarStream<CdlRisefall3methods, 4, 1>;

impl Kernel<4, 1> for CdlRisefall3methods {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_risefall3methods"];

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
