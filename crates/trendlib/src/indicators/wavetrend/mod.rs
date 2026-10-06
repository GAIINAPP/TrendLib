use crate::core::chart::whole;
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::PandasEwm;

pub const NAME: &str = "wavetrend";

pub const CHANNEL_LENGTH_DEFAULT: usize = 10;
pub const CHANNEL_LENGTH_MIN: usize = 2;
pub const CHANNEL_LENGTH_MAX: usize = 100_000;
pub const AVERAGE_LENGTH_DEFAULT: usize = 21;
pub const AVERAGE_LENGTH_MIN: usize = 2;
pub const AVERAGE_LENGTH_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub channel_length: usize,
    pub average_length: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            channel_length: CHANNEL_LENGTH_DEFAULT,
            average_length: AVERAGE_LENGTH_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    esa: PandasEwm,
    deviation: PandasEwm,
    wave: PandasEwm,
    recent: [f64; 4],
    bars: usize,
}

impl State {
    fn advance(&mut self, bar: [f64; 3]) -> Option<[f64; 2]> {
        let typical = (bar[0] + bar[1] + bar[2]) / 3.0;
        let esa = self.esa.push(typical);
        let deviation = self.deviation.push((typical - esa).abs());
        let channel = (typical - esa) / (0.015 * deviation);
        let wave = self.wave.push(channel);
        self.recent[self.bars % 4] = wave;
        self.bars += 1;
        (self.bars > LOOKBACK).then(|| [wave, self.recent.iter().sum::<f64>() / 4.0])
    }
}

/// The first bar's channel index is 0 / 0, so the wave starts on the second
/// bar and its four-bar mean on the fifth; both outputs start there.
const LOOKBACK: usize = 4;

impl Step<3, 2> for State {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 2]> {
        self.advance(bar)
    }

    fn preview(&self, bar: [f64; 3]) -> Option<[f64; 2]> {
        self.clone().advance(bar)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Wavetrend;

pub type WavetrendStream = BarStream<Wavetrend, 3, 2>;

impl Kernel<3, 2> for Wavetrend {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 2] = ["wavetrend_1", "wavetrend_2"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(
            NAME,
            "channel_length",
            params.channel_length,
            CHANNEL_LENGTH_MIN,
            CHANNEL_LENGTH_MAX,
        )?;
        whole(
            NAME,
            "average_length",
            params.average_length,
            AVERAGE_LENGTH_MIN,
            AVERAGE_LENGTH_MAX,
        )?;
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        LOOKBACK
    }

    fn state(params: &Params) -> State {
        State {
            esa: PandasEwm::with_span(params.channel_length),
            deviation: PandasEwm::with_span(params.channel_length),
            wave: PandasEwm::with_span(params.average_length),
            recent: [f64::NAN; 4],
            bars: 0,
        }
    }
}
