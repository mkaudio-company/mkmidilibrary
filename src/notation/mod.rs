//! Musical notation elements
//!
//! This module contains musical notation symbols and markings:
//! - [`KeySignature`] - Key signature (sharps/flats)
//! - [`TimeSignature`] - Time/meter signature
//! - [`Tempo`] - Tempo markings
//! - [`Dynamics`] - Dynamic markings (pp, p, mp, mf, f, ff)
//! - [`Clef`] - Clef types
//! - [`Articulation`] - Articulation markings

mod articulation;
mod clef;
mod dynamics;
mod key;
mod meter;
mod tempo;

pub use articulation::{ArticulationMark, ArticulationPlacement};
pub use clef::{Clef, ClefSign};
pub use dynamics::{Dynamics, DynamicsType, Hairpin, HairpinType};
pub use key::{Key, KeyMode, KeySignature};
pub use meter::TimeSignature;
pub use tempo::{MetronomeMark, Tempo, TempoIndication};
