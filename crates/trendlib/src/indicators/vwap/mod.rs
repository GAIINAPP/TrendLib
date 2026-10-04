use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::VwapAnchor;

pub const NAME: &str = "vwap";

/// Nanoseconds in a day, which is how a calendar date is read out of a
/// timestamp without a timezone database.
const DAY: f64 = 86_400_000_000_000.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub anchor: VwapAnchor,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            anchor: VwapAnchor::Day,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    anchor: VwapAnchor,
    session: Option<f64>,
    traded: f64,
    volume: f64,
}

fn typical(bar: [f64; 5]) -> f64 {
    (bar[0] + bar[1] + bar[2]) / 3.0
}

/// Which session a timestamp belongs to, or `None` when every bar is one
/// session.
fn session_of(anchor: VwapAnchor, timestamp: f64) -> Option<f64> {
    match anchor {
        VwapAnchor::Day => Some((timestamp / DAY).floor()),
        VwapAnchor::None => None,
    }
}

impl State {
    fn totals(&self, bar: [f64; 5]) -> (f64, f64) {
        let session = session_of(self.anchor, bar[4]);
        let carried = session == self.session;
        let (traded, volume) = if carried {
            (self.traded, self.volume)
        } else {
            (0.0, 0.0)
        };
        (traded + typical(bar) * bar[3], volume + bar[3])
    }
}

impl Step<5, 1> for State {
    fn push(&mut self, bar: [f64; 5]) -> Option<[f64; 1]> {
        let (traded, volume) = self.totals(bar);
        self.session = session_of(self.anchor, bar[4]);
        self.traded = traded;
        self.volume = volume;
        // Nothing has traded yet, so there is no average fill to report. A
        // zero would read as a price (`CONVENTIONS.md` deviation 2).
        Some([traded / volume])
    }

    fn preview(&self, bar: [f64; 5]) -> Option<[f64; 1]> {
        let (traded, volume) = self.totals(bar);
        Some([traded / volume])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Vwap;

pub type VwapStream = BarStream<Vwap, 5, 1>;

impl Kernel<5, 1> for Vwap {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 5] = ["high", "low", "close", "volume", "timestamps"];
    const OUTPUTS: [&'static str; 1] = ["vwap"];

    type Params = Params;
    type State = State;

    fn validate(_params: &Params) -> Result<(), TlError> {
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        0
    }

    fn state(params: &Params) -> State {
        State {
            anchor: params.anchor,
            session: None,
            traded: 0.0,
            volume: 0.0,
        }
    }
}
