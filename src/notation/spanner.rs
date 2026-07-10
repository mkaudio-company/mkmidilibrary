//! Generic spanner concept
//!
//! Mirrors music21's `spanner.Spanner` base class: a notational element
//! that applies to a *range* of the music rather than a single point, such
//! as a slur, a dynamics hairpin, a hammer-on/pull-off, or a trill
//! extension line. Real music21 tracks a spanner's endpoints by object
//! identity (a list of the actual `Note`/`Chord` objects it spans); this
//! crate's `Stream`/`Measure` model doesn't give elements a stable identity,
//! so a `Spanner` instead anchors its endpoints by musical position
//! (measure number + offset within the measure) via `SpannerAnchor`.
//!
//! This is deliberately built once and reused by every spanning notation
//! type: dynamics wedges (`Crescendo`/`Diminuendo`), articulation spanners
//! (`HammerOn`/`PullOff`), and expression spanners (`ArpeggioMarkSpanner`,
//! `TrillExtension`, `TremoloSpanner`).

use std::fmt;

use crate::core::Fraction;

/// A position within a musical stream: the measure it falls in (by number,
/// matching `Measure::number`) and the offset within that measure, in
/// quarter notes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SpannerAnchor {
    /// The measure number this position falls in.
    pub measure_number: u32,
    /// Offset within the measure, in quarter notes.
    pub offset: Fraction,
}

impl SpannerAnchor {
    /// Create a new anchor at a given measure number and offset.
    pub fn new(measure_number: u32, offset: Fraction) -> Self {
        Self {
            measure_number,
            offset,
        }
    }

    /// Create an anchor at the start of a measure (offset 0).
    pub fn start_of_measure(measure_number: u32) -> Self {
        Self::new(measure_number, Fraction::new(0, 1))
    }
}

impl fmt::Display for SpannerAnchor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "m{}+{}", self.measure_number, self.offset)
    }
}

/// A notational element spanning a range between two anchored positions.
/// Concrete spanning notations (hairpins, hammer-ons, trill extensions,
/// etc.) wrap a `Spanner` for their start/end anchoring rather than
/// re-implementing it.
#[derive(Debug, Clone, PartialEq)]
pub struct Spanner {
    start: SpannerAnchor,
    end: SpannerAnchor,
    /// Optional free-form type tag (e.g. "slur", "crescendo") for generic
    /// introspection/debugging.
    label: Option<String>,
}

impl Spanner {
    /// Create a new spanner between two anchored positions. Panics if
    /// `end` comes strictly before `start`.
    pub fn new(start: SpannerAnchor, end: SpannerAnchor) -> Self {
        assert!(
            end >= start,
            "spanner end {end} must not come before its start {start}"
        );
        Self {
            start,
            end,
            label: None,
        }
    }

    /// Create a new spanner with a descriptive label.
    pub fn with_label(start: SpannerAnchor, end: SpannerAnchor, label: impl Into<String>) -> Self {
        let mut spanner = Self::new(start, end);
        spanner.label = Some(label.into());
        spanner
    }

    /// Get the start anchor.
    pub fn start(&self) -> SpannerAnchor {
        self.start
    }

    /// Get the end anchor.
    pub fn end(&self) -> SpannerAnchor {
        self.end
    }

    /// Get the label, if any.
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Set the label.
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = Some(label.into());
    }

    /// Whether a given position falls within this spanner's range
    /// (inclusive of both endpoints).
    pub fn contains(&self, pos: SpannerAnchor) -> bool {
        pos >= self.start && pos <= self.end
    }

    /// Whether this spanner is confined to a single measure.
    pub fn is_single_measure(&self) -> bool {
        self.start.measure_number == self.end.measure_number
    }

    /// The number of measures this spanner crosses (inclusive), e.g. a
    /// spanner from measure 3 to measure 5 crosses 3 measures.
    pub fn measure_span(&self) -> u32 {
        self.end.measure_number - self.start.measure_number + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spanner_creation_and_contains() {
        let start = SpannerAnchor::new(1, Fraction::new(0, 1));
        let end = SpannerAnchor::new(2, Fraction::new(1, 1));
        let spanner = Spanner::new(start, end);

        assert_eq!(spanner.start(), start);
        assert_eq!(spanner.end(), end);
        assert!(spanner.contains(start));
        assert!(spanner.contains(end));
        assert!(spanner.contains(SpannerAnchor::new(1, Fraction::new(3, 1))));
        assert!(!spanner.contains(SpannerAnchor::new(3, Fraction::new(0, 1))));
    }

    #[test]
    fn test_spanner_measure_span() {
        let spanner = Spanner::new(
            SpannerAnchor::start_of_measure(3),
            SpannerAnchor::start_of_measure(5),
        );
        assert!(!spanner.is_single_measure());
        assert_eq!(spanner.measure_span(), 3);

        let single = Spanner::new(
            SpannerAnchor::new(1, Fraction::new(0, 1)),
            SpannerAnchor::new(1, Fraction::new(2, 1)),
        );
        assert!(single.is_single_measure());
        assert_eq!(single.measure_span(), 1);
    }

    #[test]
    fn test_spanner_with_label() {
        let spanner = Spanner::with_label(
            SpannerAnchor::start_of_measure(1),
            SpannerAnchor::start_of_measure(1),
            "crescendo",
        );
        assert_eq!(spanner.label(), Some("crescendo"));
    }

    #[test]
    #[should_panic(expected = "must not come before its start")]
    fn test_spanner_rejects_end_before_start() {
        Spanner::new(SpannerAnchor::start_of_measure(5), SpannerAnchor::start_of_measure(1));
    }
}
