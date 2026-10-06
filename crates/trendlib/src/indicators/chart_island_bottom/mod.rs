use crate::core::bars::BarHistory;
use crate::core::chart::whole;
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};
use crate::core::shapes::HlcState;

pub const NAME: &str = "chart_island_bottom";

pub const MAX_ISLAND_BARS_DEFAULT: usize = 10;
pub const MAX_ISLAND_BARS_MIN: usize = 1;
pub const MAX_ISLAND_BARS_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub max_island_bars: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            max_island_bars: MAX_ISLAND_BARS_DEFAULT,
        }
    }
}

pub type State = HlcState<Params>;

/// The latest gap down within the last `max_island_bars - 1` bars, and this bar
/// gapping up above both the bar before it and the bar that gapped down.
fn rule(history: &BarHistory, params: &Params) -> f64 {
    let (bar, previous) = (history.back(0), history.back(1));
    for back in 1..params.max_island_bars {
        let entry = history.back(back);
        if entry.high < history.back(back + 1).low {
            let exits = bar.low > previous.high && bar.low > entry.high && bar.close > entry.high;
            return if exits { 100.0 } else { 0.0 };
        }
    }
    0.0
}
#[derive(Clone, Copy, Debug)]
pub struct ChartIslandBottom;

pub type ChartIslandBottomStream = BarStream<ChartIslandBottom, 3, 1>;

impl Kernel<3, 1> for ChartIslandBottom {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["chart_island_bottom"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(
            NAME,
            "max_island_bars",
            params.max_island_bars,
            MAX_ISLAND_BARS_MIN,
            MAX_ISLAND_BARS_MAX,
        )?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.max_island_bars + 1
    }

    fn state(params: &Params) -> State {
        HlcState::new(
            params.max_island_bars,
            params.max_island_bars + 1,
            *params,
            rule,
        )
    }
}
