use crate::core::chart::at_least;
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::indicators::swing_index;

pub const NAME: &str = "asi";

pub const LIMIT_MOVE_DEFAULT: f64 = 0.5;
pub const LIMIT_MOVE_MIN: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub limit_move: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            limit_move: LIMIT_MOVE_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    swing: swing_index::State,
    total: f64,
}

impl Step<4, 1> for State {
    fn push(&mut self, bar: [f64; 4]) -> Option<[f64; 1]> {
        let [swing] = self.swing.push(bar)?;
        self.total += swing;
        Some([self.total])
    }

    fn preview(&self, bar: [f64; 4]) -> Option<[f64; 1]> {
        let [swing] = self.swing.preview(bar)?;
        Some([self.total + swing])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Asi;

pub type AsiStream = BarStream<Asi, 4, 1>;

impl Kernel<4, 1> for Asi {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["asi"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        at_least(NAME, "limit_move", params.limit_move, LIMIT_MOVE_MIN)?;
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        1
    }

    fn state(params: &Params) -> State {
        State {
            swing: swing_index::SwingIndex::state(&swing_index::Params {
                limit_move: params.limit_move,
            }),
            total: 0.0,
        }
    }
}
