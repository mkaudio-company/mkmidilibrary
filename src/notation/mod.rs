//! Musical notation elements
//!
//! This module contains musical notation symbols and markings:
//! - [`KeySignature`] - Key signature (sharps/flats)
//! - [`TimeSignature`] - Time/meter signature
//! - [`Tempo`] - Tempo markings
//! - [`Dynamics`] - Dynamic markings (pp, p, mp, mf, f, ff)
//! - [`Clef`] - Clef types
//! - [`ArticulationMark`] - Articulation markings

mod articulation;
mod beam;
mod clef;
mod dynamics;
mod expressions;
mod key;
mod meter;
mod scale;
mod spanner;
mod tempo;

pub use articulation::{ArticulationMark, ArticulationPlacement, HammerPullSpanner, HammerPullType};
pub use beam::{compute_beams, Beam, BeamType};
pub use clef::{Clef, ClefSign};
pub use dynamics::{dynamic_str_from_decimal, DynamicWedge, DynamicWedgeType, Dynamics, DynamicsType};
pub use expressions::{
    ArpeggioDirection, ArpeggioMark, Ornament, OrnamentKind, OrnamentSize, PedalMark,
    RehearsalMark, TextExpression, TremoloSpanner, TrillExtension, TurnDelay,
};
pub use key::{pitch_to_sharps, sharps_to_pitch, Key, KeyMode, KeySignature};
pub use meter::{MeterClassification, SenzaMisuraTimeSignature, TimeSignature};
pub use scale::Scale;
pub use spanner::{Spanner, SpannerAnchor};
pub use tempo::{MetronomeMark, Tempo, TempoIndication};
