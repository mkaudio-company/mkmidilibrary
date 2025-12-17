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
mod duration;
mod interval;
mod note;
mod pitch;
mod rest;

pub use accidental::{Accidental, Microtone};
pub use chord::{Chord, ChordQuality};
pub use duration::{Duration, DurationType, Tuplet};
pub use interval::{Interval, IntervalQuality};
pub use note::{
    Articulation, ArticulationType, Expression, ExpressionType, Lyric, Note, NoteHead, NoteHeadType,
    StemDirection, Tie, TieType, Volume,
};
pub use pitch::{Pitch, Step};
pub use rest::Rest;

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
}
