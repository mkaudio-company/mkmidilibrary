//! Real-time MIDI I/O
//!
//! This module provides cross-platform real-time MIDI input and output,
//! based on the RtMidi library design.
//!
//! # Platform Support
//! - macOS: CoreMIDI
//! - Linux: ALSA
//! - Windows: Windows Multimedia API

mod input;
mod output;
mod port;

#[cfg(target_os = "macos")]
mod coremidi_impl;

#[cfg(target_os = "linux")]
mod alsa_impl;

#[cfg(target_os = "windows")]
mod winmm_impl;

pub use input::MidiInput;
pub use output::MidiOutput;
pub use port::{Api, MidiPort};

use thiserror::Error;

/// Get the version of this crate's realtime MIDI implementation (mirrors
/// upstream RtMidi's `RtMidi::getVersion()`).
pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Errors that can occur during real-time MIDI operations
#[derive(Debug, Error)]
pub enum RtMidiError {
    #[error("no MIDI ports available")]
    NoPortsAvailable,

    #[error("invalid port number: {0}")]
    InvalidPort(usize),

    #[error("port already open")]
    PortAlreadyOpen,

    #[error("port not open")]
    PortNotOpen,

    #[error("failed to create virtual port")]
    VirtualPortError,

    #[error("system error: {0}")]
    SystemError(String),

    #[error("driver error: {0}")]
    DriverError(String),

    #[error("invalid message")]
    InvalidMessage,

    #[error("thread error: {0}")]
    ThreadError(String),

    /// A non-fatal condition (e.g. a dropped message because the polling
    /// queue is full, or an unplugged device). Unlike the other variants,
    /// this is never returned from a `Result` — it is only ever delivered
    /// via `set_error_callback`, matching upstream RtMidi's
    /// `RtMidiError::WARNING`.
    #[error("warning: {0}")]
    Warning(String),

    /// A non-fatal, low-level diagnostic condition, delivered only via
    /// `set_error_callback`. Matches upstream RtMidi's
    /// `RtMidiError::DEBUG_WARNING`.
    #[error("debug warning: {0}")]
    DebugWarning(String),
}

/// MIDI callback function type
pub type MidiCallback = Box<dyn FnMut(f64, &[u8]) + Send>;

/// Error/warning callback function type, used to report non-fatal
/// conditions (see `RtMidiError::Warning`/`DebugWarning`) that don't fit a
/// `Result` return value.
pub type RtMidiErrorCallback = Box<dyn FnMut(&RtMidiError) + Send>;

/// Configuration for MIDI input
#[derive(Debug, Clone)]
pub struct MidiInputConfig {
    /// Queue size for incoming messages (when not using callbacks)
    pub queue_size: usize,
    /// Ignore MIDI timing messages
    pub ignore_timing: bool,
    /// Ignore MIDI active sensing messages
    pub ignore_active_sensing: bool,
    /// Ignore system exclusive messages
    pub ignore_sysex: bool,
}

impl Default for MidiInputConfig {
    fn default() -> Self {
        Self {
            queue_size: 100,
            ignore_timing: true,
            ignore_active_sensing: true,
            // Matches upstream RtMidi's default (`RtMidiInData`'s
            // `ignoreFlags(7)`, which ignores sysex/timing/active-sensing
            // all by default) — sysex is ignored unless explicitly enabled.
            ignore_sysex: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_version_matches_cargo_pkg_version() {
        assert_eq!(get_version(), env!("CARGO_PKG_VERSION"));
    }
}
