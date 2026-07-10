//! Measure representation
//!
//! A Measure represents a single bar of music.

use std::fmt;

use crate::core::{Duration, Fraction, Rest};
use crate::notation::{Clef, KeySignature, TimeSignature};

use super::base::{MusicElement, Stream};

/// A single measure of music
#[derive(Debug, Clone)]
pub struct Measure {
    /// The measure number
    number: u32,
    /// Optional number suffix (e.g., "a" for measure 12a)
    number_suffix: Option<String>,
    /// The stream of music elements
    stream: Stream,
    /// Time signature (if changed in this measure)
    time_signature: Option<TimeSignature>,
    /// Key signature (if changed in this measure)
    key_signature: Option<KeySignature>,
    /// Clef (if changed in this measure)
    clef: Option<Clef>,
    /// Whether this is a pickup (anacrusis) measure
    is_pickup: bool,
    /// Explicit duration (overrides calculated)
    explicit_duration: Option<Fraction>,
    /// Whether this measure opens a repeated section (a "||:" barline).
    repeat_start: bool,
    /// Whether this measure closes a repeated section (a ":||" barline).
    repeat_end: bool,
}

impl Measure {
    /// Create a new measure
    pub fn new(number: u32) -> Self {
        Self {
            number,
            number_suffix: None,
            stream: Stream::new(),
            time_signature: None,
            key_signature: None,
            clef: None,
            is_pickup: false,
            explicit_duration: None,
            repeat_start: false,
            repeat_end: false,
        }
    }

    /// Create a pickup (anacrusis) measure
    pub fn pickup() -> Self {
        Self {
            number: 0,
            number_suffix: None,
            stream: Stream::new(),
            time_signature: None,
            key_signature: None,
            clef: None,
            is_pickup: true,
            explicit_duration: None,
            repeat_start: false,
            repeat_end: false,
        }
    }

    /// Whether this measure opens a repeated section.
    pub fn is_repeat_start(&self) -> bool {
        self.repeat_start
    }

    /// Set whether this measure opens a repeated section.
    pub fn set_repeat_start(&mut self, repeat_start: bool) {
        self.repeat_start = repeat_start;
    }

    /// Whether this measure closes a repeated section.
    pub fn is_repeat_end(&self) -> bool {
        self.repeat_end
    }

    /// Set whether this measure closes a repeated section.
    pub fn set_repeat_end(&mut self, repeat_end: bool) {
        self.repeat_end = repeat_end;
    }

    /// Get the measure number
    pub fn number(&self) -> u32 {
        self.number
    }

    /// Set the measure number
    pub fn set_number(&mut self, number: u32) {
        self.number = number;
    }

    /// Get the number suffix
    pub fn number_suffix(&self) -> Option<&str> {
        self.number_suffix.as_deref()
    }

    /// Set the number suffix
    pub fn set_number_suffix(&mut self, suffix: impl Into<String>) {
        self.number_suffix = Some(suffix.into());
    }

    /// Get the full measure number string
    pub fn measure_number_string(&self) -> String {
        match &self.number_suffix {
            Some(suffix) => format!("{}{}", self.number, suffix),
            None => self.number.to_string(),
        }
    }

    /// Get the stream
    pub fn stream(&self) -> &Stream {
        &self.stream
    }

    /// Get mutable stream
    pub fn stream_mut(&mut self) -> &mut Stream {
        &mut self.stream
    }

    /// Get elements with offsets
    pub fn elements(&self) -> &[(Fraction, MusicElement)] {
        self.stream.elements()
    }

    /// Append an element to the measure
    pub fn append(&mut self, element: MusicElement) {
        self.stream.append(element);
    }

    /// Insert an element at a specific offset
    pub fn insert(&mut self, offset: Fraction, element: MusicElement) {
        self.stream.insert(offset, element);
    }

    /// Get the time signature
    pub fn time_signature(&self) -> Option<&TimeSignature> {
        self.time_signature.as_ref()
    }

    /// Set the time signature
    pub fn set_time_signature(&mut self, ts: TimeSignature) {
        self.time_signature = Some(ts);
    }

    /// Get the key signature
    pub fn key_signature(&self) -> Option<&KeySignature> {
        self.key_signature.as_ref()
    }

    /// Set the key signature
    pub fn set_key_signature(&mut self, ks: KeySignature) {
        self.key_signature = Some(ks);
    }

    /// Get the clef, if this measure sets one explicitly.
    pub fn clef(&self) -> Option<&Clef> {
        self.clef.as_ref()
    }

    /// Set the clef.
    pub fn set_clef(&mut self, clef: Clef) {
        self.clef = Some(clef);
    }

    /// Get the explicit duration override, if one was set with
    /// `set_duration` (as opposed to one derived from a time signature).
    pub fn explicit_duration(&self) -> Option<Fraction> {
        self.explicit_duration
    }

    /// Check if this is a pickup measure
    pub fn is_pickup(&self) -> bool {
        self.is_pickup
    }

    /// Set whether this is a pickup measure
    pub fn set_pickup(&mut self, is_pickup: bool) {
        self.is_pickup = is_pickup;
    }

    /// Get the duration (in quarter lengths) based on this measure's own
    /// explicit duration/time signature only. A `Measure` has no reference
    /// to its containing `Part`, so if neither is set locally, this falls
    /// back to a bare 4/4 default rather than searching earlier measures
    /// for the prevailing time signature — which is wrong for any
    /// non-first measure in a piece not in 4/4 (most measures don't repeat
    /// the time signature explicitly). When a measure is known to live
    /// inside a `Part`, prefer `Part::measure_duration`, which performs
    /// that backward context search correctly.
    pub fn duration(&self) -> Fraction {
        if let Some(d) = self.explicit_duration {
            return d;
        }

        if let Some(ts) = &self.time_signature {
            ts.bar_duration()
        } else {
            // Default 4/4 — see caveat above; use `Part::measure_duration`
            // when context is available.
            Fraction::new(4, 1)
        }
    }

    /// Set explicit duration
    pub fn set_duration(&mut self, duration: Fraction) {
        self.explicit_duration = Some(duration);
    }

    /// Get the actual duration based on content
    pub fn content_duration(&self) -> Fraction {
        self.stream.highest_time()
    }

    /// Check if the measure is complete (filled to duration)
    pub fn is_complete(&self) -> bool {
        self.content_duration() >= self.duration()
    }

    /// Check if the measure is overfull
    pub fn is_overfull(&self) -> bool {
        self.content_duration() > self.duration()
    }

    /// Get remaining duration in the measure
    pub fn remaining_duration(&self) -> Fraction {
        let remaining = self.duration() - self.content_duration();
        if remaining < Fraction::new(0, 1) {
            Fraction::new(0, 1)
        } else {
            remaining
        }
    }

    /// Infer a plausible time signature from this measure's actual
    /// content duration: a plain `N/4` if the content is a whole number
    /// of quarter notes, else `N/8` if it's a whole number of eighths,
    /// else a denominator scaled up from the duration's own fractional
    /// form. Falls back to common time for an empty measure. Mirrors a
    /// scoped subset of music21's `Measure.bestTimeSignature`.
    pub fn best_time_signature(&self) -> TimeSignature {
        let content = self.content_duration();
        if content <= Fraction::new(0, 1) {
            return TimeSignature::common_time();
        }
        if *content.denom() == 1 {
            return TimeSignature::new(*content.numer() as u8, 4);
        }
        let eighths = content * Fraction::from(2);
        if *eighths.denom() == 1 {
            return TimeSignature::new(*eighths.numer() as u8, 8);
        }
        TimeSignature::new(*content.numer() as u8, (*content.denom() as u16 * 4) as u8)
    }

    /// Treat this (presumably underfull pickup) measure as sitting at
    /// the end of a full bar of `bar_duration`: pad it with a rest at
    /// offset 0 sized to the shortfall, shifting the existing content
    /// later by that same amount, and mark it as a pickup measure with
    /// an explicit duration of `bar_duration`. A no-op if the measure's
    /// content already fills (or exceeds) `bar_duration`. Mirrors
    /// music21's `Measure.padAsAnacrusis`.
    pub fn pad_as_anacrusis(&mut self, bar_duration: Fraction) {
        let deficit = bar_duration - self.content_duration();
        if deficit <= Fraction::new(0, 1) {
            return;
        }
        let old_elements = self.elements().to_vec();
        self.clear();
        self.insert(
            Fraction::new(0, 1),
            MusicElement::Rest(Rest::new(Duration::from_quarter_length(deficit))),
        );
        for (offset, element) in old_elements {
            self.insert(offset + deficit, element);
        }
        self.is_pickup = true;
        self.explicit_duration = Some(bar_duration);
    }

    /// The proportion (0.0-1.0, potentially over 1.0 for an overfull
    /// measure) of `bar_duration` actually filled by this measure's
    /// content. Mirrors music21's `Measure.barDurationProportion`.
    pub fn bar_duration_proportion(&self, bar_duration: Fraction) -> f64 {
        if bar_duration <= Fraction::new(0, 1) {
            return 0.0;
        }
        let content = self.content_duration();
        fraction_to_f64(content) / fraction_to_f64(bar_duration)
    }

    /// Get the number of elements
    pub fn len(&self) -> usize {
        self.stream.len()
    }

    /// Check if the measure is empty
    pub fn is_empty(&self) -> bool {
        self.stream.is_empty()
    }

    /// Clear all elements
    pub fn clear(&mut self) {
        self.stream.clear();
    }

    /// Iterate over notes
    pub fn notes(&self) -> impl Iterator<Item = &crate::core::Note> {
        self.stream.notes()
    }

    /// Iterate over chords
    pub fn chords(&self) -> impl Iterator<Item = &crate::core::Chord> {
        self.stream.chords()
    }

    /// Iterate over rests
    pub fn rests(&self) -> impl Iterator<Item = &crate::core::Rest> {
        self.stream.rests()
    }
}

impl Default for Measure {
    fn default() -> Self {
        Self::new(1)
    }
}

impl fmt::Display for Measure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Measure {} ({} elements)",
            self.measure_number_string(),
            self.len()
        )
    }
}

fn fraction_to_f64(f: Fraction) -> f64 {
    *f.numer() as f64 / *f.denom() as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Duration, Note, Pitch, Step};

    #[test]
    fn test_measure_creation() {
        let measure = Measure::new(1);
        assert_eq!(measure.number(), 1);
        assert!(measure.is_empty());
    }

    #[test]
    fn test_measure_append() {
        let mut measure = Measure::new(1);
        let note = Note::quarter(Pitch::from_parts(Step::C, Some(4), None));
        measure.append(MusicElement::Note(note));

        assert_eq!(measure.len(), 1);
    }

    #[test]
    fn test_measure_duration() {
        let mut measure = Measure::new(1);
        measure.set_time_signature(TimeSignature::new(4, 4));

        assert_eq!(measure.duration(), Fraction::new(4, 1));

        // Add elements
        let note = Note::quarter(Pitch::from_parts(Step::C, Some(4), None));
        measure.append(MusicElement::Note(note.clone()));
        measure.append(MusicElement::Note(note.clone()));

        assert_eq!(measure.content_duration(), Fraction::new(2, 1));
        assert!(!measure.is_complete());
        assert_eq!(measure.remaining_duration(), Fraction::new(2, 1));
    }

    #[test]
    fn test_measure_pickup() {
        let mut measure = Measure::pickup();
        assert!(measure.is_pickup());
        assert_eq!(measure.number(), 0);

        // Pickup measure with one beat
        let note = Note::quarter(Pitch::from_parts(Step::G, Some(4), None));
        measure.append(MusicElement::Note(note));
        measure.set_duration(Fraction::new(1, 1));

        assert!(measure.is_complete());
    }

    #[test]
    fn test_measure_suffix() {
        let mut measure = Measure::new(12);
        measure.set_number_suffix("a");

        assert_eq!(measure.measure_number_string(), "12a");
    }

    #[test]
    fn test_best_time_signature_from_content() {
        let mut whole_quarters = Measure::new(1);
        whole_quarters.append(MusicElement::Note(Note::quarter(Pitch::from_parts(
            Step::C,
            Some(4),
            None,
        ))));
        whole_quarters.append(MusicElement::Note(Note::quarter(Pitch::from_parts(
            Step::D,
            Some(4),
            None,
        ))));
        whole_quarters.append(MusicElement::Note(Note::quarter(Pitch::from_parts(
            Step::E,
            Some(4),
            None,
        ))));
        assert_eq!(
            whole_quarters.best_time_signature(),
            TimeSignature::new(3, 4)
        );

        let mut with_eighth = Measure::new(1);
        with_eighth.append(MusicElement::Note(Note::quarter(Pitch::from_parts(
            Step::C,
            Some(4),
            None,
        ))));
        with_eighth.append(MusicElement::Note(Note::eighth(Pitch::from_parts(
            Step::D,
            Some(4),
            None,
        ))));
        assert_eq!(with_eighth.best_time_signature(), TimeSignature::new(3, 8));

        assert_eq!(
            Measure::new(1).best_time_signature(),
            TimeSignature::common_time()
        );
    }

    #[test]
    fn test_pad_as_anacrusis_shifts_content_and_marks_pickup() {
        let mut measure = Measure::new(0);
        measure.append(MusicElement::Note(Note::quarter(Pitch::from_parts(
            Step::G,
            Some(4),
            None,
        ))));

        measure.pad_as_anacrusis(Fraction::new(4, 1));

        assert!(measure.is_pickup());
        assert_eq!(measure.explicit_duration(), Some(Fraction::new(4, 1)));
        assert_eq!(measure.len(), 2); // padding rest + the original note
        let (rest_offset, rest_elem) = &measure.elements()[0];
        assert_eq!(*rest_offset, Fraction::new(0, 1));
        assert!(rest_elem.is_rest());
        assert_eq!(rest_elem.quarter_length(), Fraction::new(3, 1));
        let (note_offset, note_elem) = &measure.elements()[1];
        assert_eq!(*note_offset, Fraction::new(3, 1));
        assert!(note_elem.is_note());
    }

    #[test]
    fn test_pad_as_anacrusis_is_noop_when_already_full() {
        let mut measure = Measure::new(1);
        measure.append(MusicElement::Note(Note::new(
            Pitch::from_parts(Step::C, Some(4), None),
            Duration::whole(),
        )));
        measure.pad_as_anacrusis(Fraction::new(4, 1));
        assert_eq!(measure.len(), 1); // no rest inserted
        assert!(!measure.is_pickup());
    }

    #[test]
    fn test_bar_duration_proportion() {
        let mut measure = Measure::new(1);
        measure.append(MusicElement::Note(Note::quarter(Pitch::from_parts(
            Step::C,
            Some(4),
            None,
        ))));
        measure.append(MusicElement::Note(Note::quarter(Pitch::from_parts(
            Step::D,
            Some(4),
            None,
        ))));
        assert_eq!(measure.bar_duration_proportion(Fraction::new(4, 1)), 0.5);
        assert_eq!(measure.bar_duration_proportion(Fraction::new(0, 1)), 0.0);
    }

    #[test]
    fn test_measure_clef() {
        use crate::notation::Clef;

        let mut measure = Measure::new(1);
        assert_eq!(measure.clef(), None);

        measure.set_clef(Clef::bass());
        assert_eq!(measure.clef(), Some(&Clef::bass()));
    }
}
