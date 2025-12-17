//! MIDI file I/O and message types
//!
//! This module provides support for reading and writing Standard MIDI Files (SMF),
//! as well as types for representing MIDI messages and events.

mod event;
mod file;
mod message;
mod track;
mod translate;

pub use event::MidiEvent;
pub use file::MidiFile;
pub use message::{MetaEvent, MidiMessage};
pub use track::MidiTrack;
pub use translate::{MidiToScore, ScoreToMidi};

use thiserror::Error;

/// MIDI file format type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MidiFormat {
    /// Single track
    SingleTrack = 0,
    /// Multiple tracks, synchronous
    #[default]
    MultiTrack = 1,
    /// Multiple tracks, asynchronous
    MultiSequence = 2,
}

impl TryFrom<u16> for MidiFormat {
    type Error = MidiError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MidiFormat::SingleTrack),
            1 => Ok(MidiFormat::MultiTrack),
            2 => Ok(MidiFormat::MultiSequence),
            _ => Err(MidiError::InvalidFormat(value)),
        }
    }
}

/// Errors that can occur during MIDI operations
#[derive(Debug, Error)]
pub enum MidiError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid MIDI file header")]
    InvalidHeader,

    #[error("invalid track header")]
    InvalidTrackHeader,

    #[error("invalid MIDI format: {0}")]
    InvalidFormat(u16),

    #[error("unexpected end of data")]
    UnexpectedEof,

    #[error("invalid variable-length quantity")]
    InvalidVarLen,

    #[error("invalid status byte: {0:#04x}")]
    InvalidStatus(u8),

    #[error("invalid meta event type: {0:#04x}")]
    InvalidMetaEvent(u8),

    #[error("invalid running status")]
    InvalidRunningStatus,

    #[error("track index out of bounds: {0}")]
    TrackOutOfBounds(usize),
}

/// Standard MIDI controller numbers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Controller {
    BankSelect = 0,
    ModWheel = 1,
    BreathController = 2,
    FootController = 4,
    PortamentoTime = 5,
    DataEntry = 6,
    MainVolume = 7,
    Balance = 8,
    Pan = 10,
    Expression = 11,
    EffectControl1 = 12,
    EffectControl2 = 13,
    Sustain = 64,
    Portamento = 65,
    Sostenuto = 66,
    SoftPedal = 67,
    Legato = 68,
    Hold2 = 69,
    AllSoundOff = 120,
    ResetAllControllers = 121,
    LocalControl = 122,
    AllNotesOff = 123,
}

/// General MIDI instrument names
pub const GM_INSTRUMENTS: [&str; 128] = [
    // Piano (0-7)
    "Acoustic Grand Piano", "Bright Acoustic Piano", "Electric Grand Piano",
    "Honky-tonk Piano", "Electric Piano 1", "Electric Piano 2", "Harpsichord", "Clavinet",
    // Chromatic Percussion (8-15)
    "Celesta", "Glockenspiel", "Music Box", "Vibraphone",
    "Marimba", "Xylophone", "Tubular Bells", "Dulcimer",
    // Organ (16-23)
    "Drawbar Organ", "Percussive Organ", "Rock Organ", "Church Organ",
    "Reed Organ", "Accordion", "Harmonica", "Tango Accordion",
    // Guitar (24-31)
    "Acoustic Guitar (nylon)", "Acoustic Guitar (steel)", "Electric Guitar (jazz)",
    "Electric Guitar (clean)", "Electric Guitar (muted)", "Overdriven Guitar",
    "Distortion Guitar", "Guitar Harmonics",
    // Bass (32-39)
    "Acoustic Bass", "Electric Bass (finger)", "Electric Bass (pick)", "Fretless Bass",
    "Slap Bass 1", "Slap Bass 2", "Synth Bass 1", "Synth Bass 2",
    // Strings (40-47)
    "Violin", "Viola", "Cello", "Contrabass",
    "Tremolo Strings", "Pizzicato Strings", "Orchestral Harp", "Timpani",
    // Ensemble (48-55)
    "String Ensemble 1", "String Ensemble 2", "Synth Strings 1", "Synth Strings 2",
    "Choir Aahs", "Voice Oohs", "Synth Voice", "Orchestra Hit",
    // Brass (56-63)
    "Trumpet", "Trombone", "Tuba", "Muted Trumpet",
    "French Horn", "Brass Section", "Synth Brass 1", "Synth Brass 2",
    // Reed (64-71)
    "Soprano Sax", "Alto Sax", "Tenor Sax", "Baritone Sax",
    "Oboe", "English Horn", "Bassoon", "Clarinet",
    // Pipe (72-79)
    "Piccolo", "Flute", "Recorder", "Pan Flute",
    "Blown Bottle", "Shakuhachi", "Whistle", "Ocarina",
    // Synth Lead (80-87)
    "Lead 1 (square)", "Lead 2 (sawtooth)", "Lead 3 (calliope)", "Lead 4 (chiff)",
    "Lead 5 (charang)", "Lead 6 (voice)", "Lead 7 (fifths)", "Lead 8 (bass + lead)",
    // Synth Pad (88-95)
    "Pad 1 (new age)", "Pad 2 (warm)", "Pad 3 (polysynth)", "Pad 4 (choir)",
    "Pad 5 (bowed)", "Pad 6 (metallic)", "Pad 7 (halo)", "Pad 8 (sweep)",
    // Synth Effects (96-103)
    "FX 1 (rain)", "FX 2 (soundtrack)", "FX 3 (crystal)", "FX 4 (atmosphere)",
    "FX 5 (brightness)", "FX 6 (goblins)", "FX 7 (echoes)", "FX 8 (sci-fi)",
    // Ethnic (104-111)
    "Sitar", "Banjo", "Shamisen", "Koto",
    "Kalimba", "Bagpipe", "Fiddle", "Shanai",
    // Percussive (112-119)
    "Tinkle Bell", "Agogo", "Steel Drums", "Woodblock",
    "Taiko Drum", "Melodic Tom", "Synth Drum", "Reverse Cymbal",
    // Sound Effects (120-127)
    "Guitar Fret Noise", "Breath Noise", "Seashore", "Bird Tweet",
    "Telephone Ring", "Helicopter", "Applause", "Gunshot",
];

/// Get GM instrument name by program number
pub fn gm_instrument_name(program: u8) -> &'static str {
    GM_INSTRUMENTS[program as usize & 0x7F]
}
