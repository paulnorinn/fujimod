//! Explicit physical pad connections, implemented by a board's GPIO wiring.
use crate::Cx;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Pin {
    pub port: u8,
    pub pin: u8,
}
impl Pin {
    pub fn validate(self) -> Result<(), String> {
        if self.port >= 18 || self.pin >= 16 {
            Err("invalid GPIO pad".into())
        } else {
            Ok(())
        }
    }
}

/// GPIO owns edge polarity, interrupt routing, and masked output state.
pub trait Pins: Send {
    /// None means the pad has no declared or guest-written driven level.
    fn output_level(&self, pin: Pin) -> Result<Option<bool>, String>;
    fn input(&mut self, cx: &mut Cx<'_>, pin: Pin, level: bool) -> Result<(), String>;
}
/// Observational physical input levels, with no output or processor capability.
pub trait InputPins: Send {
    fn level(&self, pin: Pin) -> Result<bool, String>;
}

/// Drive an externally supplied pad level without owning GPIO register law.
pub trait DriveInput: Send + Sync {
    fn set_input(&self, pin: Pin, high: bool);
}

/// A GPIO block whose external input levels are set directly, with no pad
/// register model and no queued request stage.
///
/// The EXR II boards present their contacts through the pad registers; the
/// X-Processor Pro sets the sampled input words directly, so the generic
/// session input path needs a block it can drive and read.
pub trait DirectPins: Send + Sync {
    fn set_input(&self, pin: Pin, high: bool) -> Result<(), String>;
    fn input(&self, pin: Pin) -> Result<bool, String>;
    /// Every bank's sampled input word.
    fn input_levels(&self) -> [u32; 16];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub pin: Pin,
    pub high: bool,
}
impl Selection {
    pub fn selected(self, pins: &dyn Pins) -> Result<bool, String> {
        Ok(pins
            .output_level(self.pin)?
            .ok_or("GPIO output has no declared or guest-written level")?
            == self.high)
    }
}
