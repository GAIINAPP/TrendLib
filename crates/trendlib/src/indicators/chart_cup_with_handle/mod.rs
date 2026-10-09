use crate::core::chart::{CupState, at_least, whole};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "chart_cup_with_handle";

pub const CUP_WINDOW_DEFAULT: usize = 65;
pub const CUP_WINDOW_MIN: usize = 2;
pub const CUP_WINDOW_MAX: usize = 100_000;
pub const HANDLE_WINDOW_DEFAULT: usize = 15;
pub const HANDLE_WINDOW_MIN: usize = 2;
pub const HANDLE_WINDOW_MAX: usize = 100_000;
pub const PIVOT_N_DEFAULT: usize = 5;
pub const PIVOT_N_MIN: usize = 1;
pub const PIVOT_N_MAX: usize = 100_000;
pub const MAX_HANDLE_RETRACE_DEFAULT: f64 = 0.5;
pub const MAX_HANDLE_RETRACE_MIN: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub cup_window: usize,
    pub handle_window: usize,
    pub pivot_n: usize,
    pub max_handle_retrace: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            cup_window: CUP_WINDOW_DEFAULT,
            handle_window: HANDLE_WINDOW_DEFAULT,
            pivot_n: PIVOT_N_DEFAULT,
            max_handle_retrace: MAX_HANDLE_RETRACE_DEFAULT,
        }
    }
}

pub type State = CupState;

#[derive(Clone, Copy, Debug)]
pub struct ChartCupWithHandle;

pub type ChartCupWithHandleStream = BarStream<ChartCupWithHandle, 3, 1>;

impl Kernel<3, 1> for ChartCupWithHandle {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["chart_cup_with_handle"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(
            NAME,
            "cup_window",
            params.cup_window,
            CUP_WINDOW_MIN,
            CUP_WINDOW_MAX,
        )?;
        whole(
            NAME,
            "handle_window",
            params.handle_window,
            HANDLE_WINDOW_MIN,
            HANDLE_WINDOW_MAX,
        )?;
        whole(NAME, "pivot_n", params.pivot_n, PIVOT_N_MIN, PIVOT_N_MAX)?;
        at_least(
            NAME,
            "max_handle_retrace",
            params.max_handle_retrace,
            MAX_HANDLE_RETRACE_MIN,
        )?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        CupState::lookback(params.cup_window, params.handle_window)
    }

    fn state(params: &Params) -> State {
        CupState::new(
            false,
            params.cup_window,
            params.handle_window,
            params.pivot_n,
            params.max_handle_retrace,
        )
    }
}
