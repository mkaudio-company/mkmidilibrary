//! MKMidiLibrary - A Rust library for music scoring and MIDI
//!
//! This library provides comprehensive support for:
//! - Music representation (notes, chords, durations, pitches)
//! - MIDI file I/O (Standard MIDI File format)
//! - Real-time MIDI input/output
//! - Score-MIDI conversion
//!
//! # Example
//!
//! ```rust
//! use mkmidilibrary::core::{Note, Pitch, Duration, Chord};
//! use mkmidilibrary::midi::MidiFile;
//!
//! // Create a note
//! let pitch = Pitch::new("C4").unwrap();
//! let duration = Duration::quarter();
//! let note = Note::new(pitch, duration);
//!
//! // Create a chord
//! let chord = Chord::major_triad(Pitch::new("C4").unwrap());
//!
//! // Create a new MIDI file
//! let midi = MidiFile::new();
//! ```

pub mod core;
pub mod midi;
pub mod notation;
pub mod stream;

#[cfg(feature = "realtime")]
pub mod realtime;

#[cfg(feature = "graphics")]
pub mod render;

pub mod analysis;

// Re-exports for convenience
pub use core::{Chord, Duration, Interval, Note, Pitch, Rest};
pub use midi::{MidiEvent, MidiFile, MidiMessage, MidiTrack};
pub use notation::{Clef, Dynamics, KeySignature, Tempo, TimeSignature};
pub use stream::{Measure, Part, Score, Stream, Voice};

#[cfg(feature = "realtime")]
pub use realtime::{MidiInput, MidiOutput, MidiPort};

#[cfg(feature = "graphics")]
pub use render::{RenderConfig, ScoreElement, ScoreRenderer};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::core::{
        Accidental, Chord, Duration, DurationType, Interval, Note, Pitch, Rest, Step,
    };
    pub use crate::midi::{MidiEvent, MidiFile, MidiMessage, MidiTrack};
    pub use crate::notation::{Clef, Dynamics, KeySignature, Tempo, TimeSignature};
    pub use crate::stream::{Measure, Part, Score, Stream, Voice};

    #[cfg(feature = "realtime")]
    pub use crate::realtime::{MidiInput, MidiOutput, MidiPort};

    #[cfg(feature = "graphics")]
    pub use crate::render::{RenderConfig, ScoreElement, ScoreRenderer};
}
