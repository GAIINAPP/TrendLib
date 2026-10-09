use crate::core::chart::{at_least, whole};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};
use crate::core::shapes::{Harmonic, HarmonicState};

pub const NAME: &str = "harmonic_butterfly_bearish";

pub const PERIOD_DEFAULT: usize = 100;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;
pub const PIVOT_N_DEFAULT: usize = 5;
pub const PIVOT_N_MIN: usize = 1;
pub const PIVOT_N_MAX: usize = 100_000;
pub const FIB_TOL_DEFAULT: f64 = 0.06;
pub const FIB_TOL_MIN: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub period: usize,
    pub pivot_n: usize,
    pub fib_tol: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
            pivot_n: PIVOT_N_DEFAULT,
            fib_tol: FIB_TOL_DEFAULT,
        }
    }
}

pub type State = HarmonicState;

#[derive(Clone, Copy, Debug)]
pub struct HarmonicButterflyBearish;

pub type HarmonicButterflyBearishStream = BarStream<HarmonicButterflyBearish, 3, 1>;

impl Kernel<3, 1> for HarmonicButterflyBearish {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["harmonic_butterfly_bearish"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(NAME, "period", params.period, PERIOD_MIN, PERIOD_MAX)?;
        whole(NAME, "pivot_n", params.pivot_n, PIVOT_N_MIN, PIVOT_N_MAX)?;
        at_least(NAME, "fib_tol", params.fib_tol, FIB_TOL_MIN)?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.period
    }

    fn state(params: &Params) -> State {
        HarmonicState::new(
            Harmonic::Xabcd {
                ab_xa: 0.786,
                bc_ab: (0.382, 0.886),
                cd_bc: (1.618, 2.24),
                ad_xa: 1.272,
            },
            false,
            params.period,
            params.pivot_n,
            params.fib_tol,
        )
    }
}
