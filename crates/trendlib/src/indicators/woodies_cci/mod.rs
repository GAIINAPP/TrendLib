use crate::core::chart::whole;
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::indicators::cci;

pub const NAME: &str = "woodies_cci";

pub const CCI_PERIOD_DEFAULT: usize = 14;
pub const CCI_PERIOD_MIN: usize = 2;
pub const CCI_PERIOD_MAX: usize = 100_000;
pub const TURBO_PERIOD_DEFAULT: usize = 6;
pub const TURBO_PERIOD_MIN: usize = 2;
pub const TURBO_PERIOD_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub cci_period: usize,
    pub turbo_period: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            cci_period: CCI_PERIOD_DEFAULT,
            turbo_period: TURBO_PERIOD_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    main: cci::State,
    turbo: cci::State,
}

impl Step<3, 2> for State {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 2]> {
        let main = self.main.push(bar);
        let turbo = self.turbo.push(bar);
        Some([main?[0], turbo?[0]])
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 2]> {
        let [main] = self.main.preview(bar)?;
        let [turbo] = self.turbo.preview(bar)?;
        Some([main, turbo])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WoodiesCci;

pub type WoodiesCciStream = BarStream<WoodiesCci, 3, 2>;

impl Kernel<3, 2> for WoodiesCci {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 2] = ["woodies_cci", "woodies_cci_turbo"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(
            NAME,
            "cci_period",
            params.cci_period,
            CCI_PERIOD_MIN,
            CCI_PERIOD_MAX,
        )?;
        whole(
            NAME,
            "turbo_period",
            params.turbo_period,
            TURBO_PERIOD_MIN,
            TURBO_PERIOD_MAX,
        )?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.cci_period.max(params.turbo_period) - 1
    }

    fn state(params: &Params) -> State {
        State {
            main: cci::Cci::state(&cci::Params {
                period: params.cci_period,
            }),
            turbo: cci::Cci::state(&cci::Params {
                period: params.turbo_period,
            }),
        }
    }
}
