//! Stream hierarchy for music containers
//!
//! Streams are ordered collections of music elements with timing information.
//! The hierarchy mirrors music21's structure:
//! - [`Stream`] - Base container for music elements
//! - [`Voice`] - A single voice within a measure
//! - [`Measure`] - A single measure of music
//! - [`Part`] - A single instrument part
//! - [`Score`] - A complete musical score

mod base;
mod measure;
mod part;
mod score;
mod voice;

pub use base::{MusicElement, Stream, StreamElement};
pub use measure::Measure;
pub use part::Part;
pub use score::{Metadata, Score};
pub use voice::Voice;
