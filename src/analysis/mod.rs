//! Music analysis tools
//!
//! This module provides tools for analyzing musical content:
//! - Chord identification and analysis
//! - Key detection
//! - Harmonic analysis

mod chord_analysis;
mod discrete;
mod floating_key;
mod melody;
mod reduce_chords;

pub use chord_analysis::{roman_numeral_from_chord, ChordAnalyzer, ChordQuality, RomanNumeral};
pub use discrete::{
    analyze_part, analyze_part_with_certainty, find_key, pitch_class_distribution, tonal_certainty,
    KeyAnalysisResult, KeyFindingAlgorithm,
};
pub use floating_key::{analyze_floating_key, detect_modulations, WindowedKeyResult};
pub use melody::{ambitus, melodic_interval_diversity};
pub use reduce_chords::ChordReducer;

/// The result of `Part::analyze`'s string-dispatched analysis methods.
#[derive(Debug, Clone, PartialEq)]
pub enum PartAnalysisResult {
    Key(KeyAnalysisResult),
    Ambitus(Option<crate::core::Interval>),
    MelodicIntervalDiversity(f64),
}

/// Backing implementation for `Part::analyze(method)`: dispatches on a
/// method name to the matching analysis routine. Mirrors music21's
/// `Stream.analyze` dispatcher for the analyses implemented in this
/// crate (`"key"`, `"ambitus"`/`"range"`, `"melodicIntervalDiversity"`).
pub fn analyze_part_by_method(
    part: &crate::stream::Part,
    method: &str,
) -> Option<PartAnalysisResult> {
    match method {
        "key" => Some(PartAnalysisResult::Key(analyze_part(
            part,
            KeyFindingAlgorithm::KrumhanslSchmuckler,
        ))),
        "ambitus" | "range" => Some(PartAnalysisResult::Ambitus(ambitus(part))),
        "melodicIntervalDiversity" => Some(PartAnalysisResult::MelodicIntervalDiversity(
            melodic_interval_diversity(part),
        )),
        _ => None,
    }
}
