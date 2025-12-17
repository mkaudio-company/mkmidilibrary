//! Measure representation
//!
//! A Measure represents a single bar of music.

use std::fmt;

use crate::core::Fraction;
use crate::notation::{KeySignature, TimeSignature};

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
    /// Whether this is a pickup (anacrusis) measure
    is_pickup: bool,
    /// Explicit duration (overrides calculated)
    explicit_duration: Option<Fraction>,
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
            is_pickup: false,
            explicit_duration: None,
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
            is_pickup: true,
            explicit_duration: None,
        }
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

    /// Check if this is a pickup measure
    pub fn is_pickup(&self) -> bool {
        self.is_pickup
    }

    /// Set whether this is a pickup measure
    pub fn set_pickup(&mut self, is_pickup: bool) {
        self.is_pickup = is_pickup;
    }

    /// Get the duration (in quarter lengths)
    pub fn duration(&self) -> Fraction {
        if let Some(d) = self.explicit_duration {
            return d;
        }

        if let Some(ts) = &self.time_signature {
            ts.bar_duration()
        } else {
            // Default 4/4
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
        write!(f, "Measure {} ({} elements)", self.measure_number_string(), self.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Note, Pitch, Step};

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
}
