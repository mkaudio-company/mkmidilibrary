//! Core music primitives
//!
//! This module contains the fundamental building blocks for representing music:
//! - [`Pitch`] - Musical pitch with step, octave, and accidental
//! - [`Duration`] - Rhythmic duration in quarter note lengths
//! - [`Note`] - A pitch with a duration
//! - [`Chord`] - Multiple simultaneous pitches
//! - [`Rest`] - Silence with a duration
//! - [`Interval`] - Distance between two pitches

mod accidental;
mod chord;
mod chord_tables;
mod duration;
mod interval;
mod note;
mod pitch;
mod rest;

pub use accidental::{Accidental, AccidentalDisplay, AccidentalDisplayType, Microtone};
pub use chord::{Chord, ChordQuality};
pub use duration::{Duration, DurationTuple, DurationType, GraceDuration, Tuplet, TupletFixer};
pub use interval::{
    add, convert_diatonic_number_to_step, get_absolute_higher_note, get_absolute_lower_note,
    get_written_higher_note, get_written_lower_note, notes_to_interval, subtract, Interval,
    IntervalQuality,
};
pub use note::{
    is_composite_lyric_set, Articulation, ArticulationMark, Expression, ExpressionType, Lyric,
    Note, NoteHead, NoteHeadType, StemDirection, Tie, TieType, Unpitched, Volume,
};
pub use pitch::{update_accidental_display, Pitch, Step};
pub use rest::{FullMeasureRest, Rest};

use num::rational::Ratio;
use thiserror::Error;

/// Fraction type used for precise rhythmic values
pub type Fraction = Ratio<i64>;

/// Errors that can occur when parsing music elements
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ParseError {
    #[error("invalid pitch string: {0}")]
    InvalidPitch(String),

    #[error("invalid step: {0}")]
    InvalidStep(String),

    #[error("invalid octave: {0}")]
    InvalidOctave(String),

    #[error("invalid accidental: {0}")]
    InvalidAccidental(String),

    #[error("invalid duration type: {0}")]
    InvalidDurationType(String),

    #[error("invalid interval: {0}")]
    InvalidInterval(String),

    #[error("invalid time signature: {0}")]
    InvalidTimeSignature(String),
}
