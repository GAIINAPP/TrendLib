use crate::core::chart::whole;
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::RollingExtreme;

pub const NAME: &str = "ichimoku";

pub const TENKAN_PERIOD_DEFAULT: usize = 9;
pub const TENKAN_PERIOD_MIN: usize = 1;
pub const TENKAN_PERIOD_MAX: usize = 100_000;
pub const KIJUN_PERIOD_DEFAULT: usize = 26;
pub const KIJUN_PERIOD_MIN: usize = 1;
pub const KIJUN_PERIOD_MAX: usize = 100_000;
pub const SENKOU_PERIOD_DEFAULT: usize = 52;
pub const SENKOU_PERIOD_MIN: usize = 1;
pub const SENKOU_PERIOD_MAX: usize = 100_000;
pub const DISPLACEMENT_DEFAULT: usize = 26;
pub const DISPLACEMENT_MIN: usize = 1;
pub const DISPLACEMENT_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub tenkan_period: usize,
    pub kijun_period: usize,
    pub senkou_period: usize,
    pub displacement: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            tenkan_period: TENKAN_PERIOD_DEFAULT,
            kijun_period: KIJUN_PERIOD_DEFAULT,
            senkou_period: SENKOU_PERIOD_DEFAULT,
            displacement: DISPLACEMENT_DEFAULT,
        }
    }
}

/// The midpoint of the highest high and the lowest low over a window.
#[derive(Clone, Debug)]
struct Midpoint {
    highest: RollingExtreme,
    lowest: RollingExtreme,
}

impl Midpoint {
    fn new(period: usize) -> Self {
        Self {
            highest: RollingExtreme::highest(period),
            lowest: RollingExtreme::lowest(period),
        }
    }

    fn push(&mut self, high: f64, low: f64) -> Option<f64> {
        let top = self.highest.push(high);
        let bottom = self.lowest.push(low);
        Some((top? + bottom?) / 2.0)
    }
}

#[derive(Clone, Debug)]
pub struct State {
    tenkan: Midpoint,
    kijun: Midpoint,
    senkou: Midpoint,
    /// The two leading spans as computed, held until they are drawn
    /// `displacement` bars later. A span not yet defined is held as NaN.
    ahead: Box<[[f64; 2]]>,
    head: usize,
    lookback: usize,
    bars: usize,
}

impl State {
    fn advance(&mut self, bar: [f64; 2]) -> Option<[f64; 4]> {
        let tenkan = self.tenkan.push(bar[0], bar[1]);
        let kijun = self.kijun.push(bar[0], bar[1]);
        let senkou = self.senkou.push(bar[0], bar[1]);
        let span_a = match (tenkan, kijun) {
            (Some(t), Some(k)) => (t + k) / 2.0,
            _ => f64::NAN,
        };
        let span_b = senkou.unwrap_or(f64::NAN);
        self.ahead[self.head] = [span_a, span_b];
        self.head = (self.head + 1) % self.ahead.len();
        let now = self.bars;
        self.bars += 1;
        if now < self.lookback {
            return None;
        }
        // The slot about to be overwritten is the one written `displacement`
        // bars ago.
        let drawn = self.ahead[self.head];
        Some([tenkan?, kijun?, drawn[0], drawn[1]])
    }
}

impl Step<2, 4> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 4]> {
        self.advance(bar)
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 4]> {
        self.clone().advance(bar)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Ichimoku;

pub type IchimokuStream = BarStream<Ichimoku, 2, 4>;

impl Kernel<2, 4> for Ichimoku {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["high", "low"];
    const OUTPUTS: [&'static str; 4] = [
        "ichimoku_tenkan",
        "ichimoku_kijun",
        "ichimoku_senkou_a",
        "ichimoku_senkou_b",
    ];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(
            NAME,
            "tenkan_period",
            params.tenkan_period,
            TENKAN_PERIOD_MIN,
            TENKAN_PERIOD_MAX,
        )?;
        whole(
            NAME,
            "kijun_period",
            params.kijun_period,
            KIJUN_PERIOD_MIN,
            KIJUN_PERIOD_MAX,
        )?;
        whole(
            NAME,
            "senkou_period",
            params.senkou_period,
            SENKOU_PERIOD_MIN,
            SENKOU_PERIOD_MAX,
        )?;
        whole(
            NAME,
            "displacement",
            params.displacement,
            DISPLACEMENT_MIN,
            DISPLACEMENT_MAX,
        )?;
        Ok(())
    }

    /// Every output starts on the same row, the first the displaced spans
    /// reach: the longest window's warm-up plus the displacement.
    fn lookback(params: &Params) -> usize {
        let longest = params
            .tenkan_period
            .max(params.kijun_period)
            .max(params.senkou_period);
        longest - 1 + params.displacement
    }

    fn state(params: &Params) -> State {
        State {
            tenkan: Midpoint::new(params.tenkan_period),
            kijun: Midpoint::new(params.kijun_period),
            senkou: Midpoint::new(params.senkou_period),
            ahead: vec![[f64::NAN; 2]; params.displacement + 1].into_boxed_slice(),
            head: 0,
            lookback: Self::lookback(params),
            bars: 0,
        }
    }
}
