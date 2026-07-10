//! Score representation
//!
//! A Score is a complete musical composition containing multiple parts.

use std::fmt;

use crate::core::{Chord, Duration, Fraction, Interval, Pitch, Rest};
use crate::notation::{KeySignature, Tempo, TimeSignature};

use super::base::MusicElement;
use super::measure::Measure;
use super::part::{Instrument, Part};

/// Score metadata
#[derive(Debug, Clone, Default)]
pub struct Metadata {
    /// Title
    pub title: Option<String>,
    /// Composer
    pub composer: Option<String>,
    /// Arranger
    pub arranger: Option<String>,
    /// Lyricist
    pub lyricist: Option<String>,
    /// Copyright
    pub copyright: Option<String>,
    /// Movement title
    pub movement_title: Option<String>,
    /// Movement number
    pub movement_number: Option<u32>,
    /// Work title
    pub work_title: Option<String>,
    /// Work number (opus)
    pub opus: Option<String>,
    /// Date of composition
    pub date: Option<String>,
}

impl Metadata {
    /// Create new empty metadata
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the composer
    pub fn with_composer(mut self, composer: impl Into<String>) -> Self {
        self.composer = Some(composer.into());
        self
    }
}

/// A complete musical score
#[derive(Debug, Clone, Default)]
pub struct Score {
    /// Parts in this score
    parts: Vec<Part>,
    /// Score metadata
    metadata: Metadata,
    /// Initial tempo
    tempo: Option<Tempo>,
    /// Initial time signature
    time_signature: Option<TimeSignature>,
    /// Initial key signature
    key_signature: Option<KeySignature>,
}

impl Score {
    /// Create a new empty score
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a score with title
    pub fn with_title(title: impl Into<String>) -> Self {
        Self {
            metadata: Metadata::new().with_title(title),
            ..Default::default()
        }
    }

    /// Get the metadata
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// Get mutable metadata
    pub fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.metadata
    }

    /// Set the metadata
    pub fn set_metadata(&mut self, metadata: Metadata) {
        self.metadata = metadata;
    }

    /// Get the title
    pub fn title(&self) -> Option<&str> {
        self.metadata.title.as_deref()
    }

    /// Set the title
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.metadata.title = Some(title.into());
    }

    /// Get the composer
    pub fn composer(&self) -> Option<&str> {
        self.metadata.composer.as_deref()
    }

    /// Set the composer
    pub fn set_composer(&mut self, composer: impl Into<String>) {
        self.metadata.composer = Some(composer.into());
    }

    /// Get all parts
    pub fn parts(&self) -> &[Part] {
        &self.parts
    }

    /// Get mutable parts
    pub fn parts_mut(&mut self) -> &mut Vec<Part> {
        &mut self.parts
    }

    /// Get a specific part
    pub fn part(&self, index: usize) -> Option<&Part> {
        self.parts.get(index)
    }

    /// Get a mutable specific part
    pub fn part_mut(&mut self, index: usize) -> Option<&mut Part> {
        self.parts.get_mut(index)
    }

    /// Get part by ID
    pub fn part_by_id(&self, id: &str) -> Option<&Part> {
        self.parts.iter().find(|p| p.id() == Some(id))
    }

    /// Add a part
    pub fn add_part(&mut self, part: Part) {
        self.parts.push(part);
    }

    /// Insert a part at index
    pub fn insert_part(&mut self, index: usize, part: Part) {
        self.parts.insert(index, part);
    }

    /// Remove a part
    pub fn remove_part(&mut self, index: usize) -> Option<Part> {
        if index < self.parts.len() {
            Some(self.parts.remove(index))
        } else {
            None
        }
    }

    /// Get the number of parts
    pub fn num_parts(&self) -> usize {
        self.parts.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    /// Get the tempo
    pub fn tempo(&self) -> Option<&Tempo> {
        self.tempo.as_ref()
    }

    /// Set the tempo
    pub fn set_tempo(&mut self, tempo: Tempo) {
        self.tempo = Some(tempo);
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

    /// Get the number of measures (from the longest part)
    pub fn num_measures(&self) -> usize {
        self.parts
            .iter()
            .map(|p| p.num_measures())
            .max()
            .unwrap_or(0)
    }

    /// Get total duration in quarter lengths
    pub fn duration(&self) -> crate::core::Fraction {
        self.parts
            .iter()
            .map(|p| p.duration())
            .max()
            .unwrap_or(crate::core::Fraction::new(0, 1))
    }

    /// Iterate over all notes in the score
    pub fn notes(&self) -> impl Iterator<Item = &crate::core::Note> {
        self.parts.iter().flat_map(|p| p.notes())
    }

    /// Iterate over all chords in the score
    pub fn chords(&self) -> impl Iterator<Item = &crate::core::Chord> {
        self.parts.iter().flat_map(|p| p.chords())
    }

    /// A copy of this score with every part transposed by `interval`
    /// (see `Part::transpose`).
    pub fn transpose(&self, interval: &Interval) -> Score {
        let mut result = self.clone();
        for part in &mut result.parts {
            *part = part.transpose(interval);
        }
        result
    }

    /// A copy of this score with every part's content augmented/
    /// diminished by `scalar` (see `Part::augment_or_diminish`).
    pub fn augment_or_diminish(&self, scalar: Fraction) -> Score {
        let mut result = self.clone();
        for part in &mut result.parts {
            *part = part.augment_or_diminish(scalar);
        }
        result
    }

    /// Every part's instrument, in part order (only parts that actually
    /// have one set). Mirrors music21's `Stream.getInstruments`.
    pub fn get_instruments(&self) -> Vec<&Instrument> {
        self.parts.iter().filter_map(|p| p.instrument()).collect()
    }

    /// Ensure all parts have the same number of measures
    pub fn pad_measures(&mut self) {
        let max_measures = self.num_measures();
        for part in &mut self.parts {
            part.ensure_measures(max_measures);
        }
    }

    /// Create a new part and return its index
    pub fn create_part(&mut self, name: impl Into<String>) -> usize {
        let part = Part::with_name(name);
        self.parts.push(part);
        self.parts.len() - 1
    }

    /// Reduce this score's parts into a single chordal-reduction `Part`:
    /// at every point where any part's content starts or ends, the
    /// pitches sounding across *all* parts at that instant (from `Note`s
    /// and `Chord`s; `Rest`s contribute nothing) are combined into one
    /// `Chord` covering that interval (or a `Rest`, if nothing is
    /// sounding). Mirrors a scoped subset of music21's `Stream.chordify`:
    /// the result is a single measure spanning the whole piece (this
    /// doesn't yet re-split the result at the original measure/barline
    /// boundaries), and it takes its time signature from the first part's
    /// first measure, if any.
    pub fn chordify(&self) -> Part {
        let combined: Vec<(Fraction, MusicElement)> =
            self.parts.iter().flat_map(|p| p.flatten()).collect();
        let time_signature = self
            .parts
            .first()
            .and_then(|p| p.measures().first())
            .and_then(|m| m.time_signature())
            .copied();

        let mut result = Part::with_name("Chordified");
        result.add_measure(chordal_reduction_measure(&combined, 1, time_signature));
        result
    }

    /// Combine a set of parts' simultaneous content into a single `Part`,
    /// like `chordify`, but preserving measure-by-measure structure
    /// instead of collapsing the whole piece into one giant measure:
    /// output measure `i` is built only from input measure `i` of each
    /// given part (so all parts must share the same number of measures —
    /// pad with `Part::ensure_measures` first if they don't), using
    /// within-measure (not absolute) offsets for the boundary-slicing.
    /// Mirrors a scoped subset of music21's `Score.implode`.
    pub fn implode(parts: &[&Part]) -> Part {
        let num_measures = parts.iter().map(|p| p.num_measures()).min().unwrap_or(0);
        let mut result = Part::with_name("Imploded");

        for mi in 0..num_measures {
            let combined: Vec<(Fraction, MusicElement)> = parts
                .iter()
                .filter_map(|p| p.measure(mi))
                .flat_map(|m| m.elements().iter().cloned())
                .collect();
            let time_signature = parts
                .first()
                .and_then(|p| p.measure(mi))
                .and_then(|m| m.time_signature())
                .copied();

            result.add_measure(chordal_reduction_measure(
                &combined,
                mi as u32 + 1,
                time_signature,
            ));
        }

        result
    }
}

/// Shared slicing logic for `chordify`/`implode`: slice `combined`
/// (offset-tagged elements, all drawn from the same "local" offset
/// space) at every point where any element starts or ends, and build a
/// single `Measure` numbered `measure_number` where each slice becomes a
/// `Chord` of every pitch sounding across `combined` during that slice
/// (or a `Rest`, if nothing is sounding).
fn chordal_reduction_measure(
    combined: &[(Fraction, MusicElement)],
    measure_number: u32,
    time_signature: Option<TimeSignature>,
) -> Measure {
    let mut boundaries: Vec<Fraction> = Vec::new();
    for (offset, element) in combined {
        boundaries.push(*offset);
        boundaries.push(*offset + element.quarter_length());
    }
    boundaries.sort();
    boundaries.dedup();

    let mut measure = Measure::new(measure_number);
    if let Some(ts) = time_signature {
        measure.set_time_signature(ts);
    }

    for window in boundaries.windows(2) {
        let (start, end) = (window[0], window[1]);
        let mut pitches: Vec<Pitch> = Vec::new();
        for (offset, element) in combined {
            let elem_end = *offset + element.quarter_length();
            if *offset <= start && elem_end >= end {
                match element {
                    MusicElement::Note(n) => pitches.push(n.pitch().clone()),
                    MusicElement::Chord(c) => pitches.extend(c.pitches().into_iter().cloned()),
                    MusicElement::Rest(_) => {}
                }
            }
        }

        let duration = Duration::from_quarter_length(end - start);
        if pitches.is_empty() {
            measure.insert(start, MusicElement::Rest(Rest::new(duration)));
        } else {
            measure.insert(
                start,
                MusicElement::Chord(Chord::from_pitches(pitches, duration)),
            );
        }
    }

    measure
}

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let title = self.metadata.title.as_deref().unwrap_or("Untitled");
        write!(
            f,
            "Score '{}' ({} parts, {} measures)",
            title,
            self.parts.len(),
            self.num_measures()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream::Measure;

    #[test]
    fn test_score_creation() {
        let score = Score::with_title("Symphony No. 5");
        assert_eq!(score.title(), Some("Symphony No. 5"));
        assert!(score.is_empty());
    }

    #[test]
    fn test_score_parts() {
        let mut score = Score::new();
        let mut part = Part::with_name("Violin");
        part.add_measure(Measure::new(1));
        score.add_part(part);

        assert_eq!(score.num_parts(), 1);
        assert_eq!(score.num_measures(), 1);
    }

    #[test]
    fn test_score_metadata() {
        let mut score = Score::new();
        score.set_title("Moonlight Sonata");
        score.set_composer("Ludwig van Beethoven");

        assert_eq!(score.title(), Some("Moonlight Sonata"));
        assert_eq!(score.composer(), Some("Ludwig van Beethoven"));
    }

    #[test]
    fn test_score_create_part() {
        let mut score = Score::new();
        let idx = score.create_part("Flute");

        assert_eq!(idx, 0);
        assert_eq!(score.part(0).unwrap().name(), Some("Flute"));
    }

    #[test]
    fn test_chordify_combines_simultaneous_notes_across_parts() {
        use crate::core::{Note, Pitch, Step};
        use crate::stream::MusicElement;

        // Part 1: C4 quarter, then E4 quarter.
        let mut part1 = Part::with_name("Soprano");
        let mut m1a = Measure::new(1);
        m1a.set_time_signature(TimeSignature::new(4, 4));
        m1a.insert(
            Fraction::new(0, 1),
            MusicElement::Note(Note::quarter(Pitch::from_parts(Step::C, Some(4), None))),
        );
        m1a.insert(
            Fraction::new(1, 1),
            MusicElement::Note(Note::quarter(Pitch::from_parts(Step::E, Some(4), None))),
        );
        part1.add_measure(m1a);

        // Part 2: a half note G3 spanning both.
        let mut part2 = Part::with_name("Bass");
        let mut m1b = Measure::new(1);
        m1b.insert(
            Fraction::new(0, 1),
            MusicElement::Note(Note::half(Pitch::from_parts(Step::G, Some(3), None))),
        );
        part2.add_measure(m1b);

        let mut score = Score::new();
        score.add_part(part1);
        score.add_part(part2);

        let chordified = score.chordify();
        let measure = chordified.measure(0).unwrap();
        let elements = measure.elements();

        // Two slices: [0,1) has C4+G3, [1,2) has E4+G3.
        assert_eq!(elements.len(), 2);

        let (offset0, elem0) = &elements[0];
        assert_eq!(*offset0, Fraction::new(0, 1));
        let chord0 = elem0.as_chord().unwrap();
        let names0: Vec<String> = chord0.pitches().iter().map(|p| p.name()).collect();
        assert!(names0.contains(&"C".to_string()));
        assert!(names0.contains(&"G".to_string()));

        let (offset1, elem1) = &elements[1];
        assert_eq!(*offset1, Fraction::new(1, 1));
        let chord1 = elem1.as_chord().unwrap();
        let names1: Vec<String> = chord1.pitches().iter().map(|p| p.name()).collect();
        assert!(names1.contains(&"E".to_string()));
        assert!(names1.contains(&"G".to_string()));
    }

    #[test]
    fn test_chordify_produces_rest_when_nothing_sounding() {
        use crate::core::{Duration, Note, Pitch, Step};
        use crate::stream::MusicElement;

        let mut part = Part::with_name("Solo");
        let mut m1 = Measure::new(1);
        // A note, then a gap (nothing inserted from offset 1 to 2), then
        // another note — chordify must fill the gap with a Rest rather
        // than silently omitting it.
        m1.insert(
            Fraction::new(0, 1),
            MusicElement::Note(Note::quarter(Pitch::from_parts(Step::C, Some(4), None))),
        );
        m1.insert(
            Fraction::new(2, 1),
            MusicElement::Note(Note::new(
                Pitch::from_parts(Step::D, Some(4), None),
                Duration::quarter(),
            )),
        );
        part.add_measure(m1);

        let mut score = Score::new();
        score.add_part(part);

        let chordified = score.chordify();
        let measure = chordified.measure(0).unwrap();
        let elements = measure.elements();

        assert_eq!(elements.len(), 3);
        assert!(elements[0].1.is_chord());
        assert!(elements[1].1.is_rest());
        assert_eq!(elements[1].0, Fraction::new(1, 1));
        assert!(elements[2].1.is_chord());
    }

    #[test]
    fn test_implode_preserves_measure_structure_unlike_chordify() {
        use crate::core::{Note, Pitch, Step};
        use crate::stream::MusicElement;

        // Two 2-measure parts.
        let mut part1 = Part::with_name("Soprano");
        let mut p1m1 = Measure::new(1);
        p1m1.set_time_signature(TimeSignature::new(4, 4));
        p1m1.insert(
            Fraction::new(0, 1),
            MusicElement::Note(Note::quarter(Pitch::from_parts(Step::C, Some(4), None))),
        );
        part1.add_measure(p1m1);
        let mut p1m2 = Measure::new(2);
        p1m2.insert(
            Fraction::new(0, 1),
            MusicElement::Note(Note::quarter(Pitch::from_parts(Step::D, Some(4), None))),
        );
        part1.add_measure(p1m2);

        let mut part2 = Part::with_name("Alto");
        let mut p2m1 = Measure::new(1);
        p2m1.insert(
            Fraction::new(0, 1),
            MusicElement::Note(Note::quarter(Pitch::from_parts(Step::G, Some(3), None))),
        );
        part2.add_measure(p2m1);
        let mut p2m2 = Measure::new(2);
        p2m2.insert(
            Fraction::new(0, 1),
            MusicElement::Note(Note::quarter(Pitch::from_parts(Step::A, Some(3), None))),
        );
        part2.add_measure(p2m2);

        let imploded = Score::implode(&[&part1, &part2]);

        // Unlike chordify (one giant measure), implode keeps 2 measures.
        assert_eq!(imploded.num_measures(), 2);

        let m1_chord = imploded.measure(0).unwrap().elements()[0]
            .1
            .as_chord()
            .unwrap();
        let names1: Vec<String> = m1_chord.pitches().iter().map(|p| p.name()).collect();
        assert!(names1.contains(&"C".to_string()));
        assert!(names1.contains(&"G".to_string()));

        let m2_chord = imploded.measure(1).unwrap().elements()[0]
            .1
            .as_chord()
            .unwrap();
        let names2: Vec<String> = m2_chord.pitches().iter().map(|p| p.name()).collect();
        assert!(names2.contains(&"D".to_string()));
        assert!(names2.contains(&"A".to_string()));
    }

    #[test]
    fn test_get_instruments_skips_unset_parts() {
        use super::super::part::Instrument;

        let mut score = Score::new();
        let mut violin = Part::with_name("Violin I");
        violin.set_instrument(Instrument::violin());
        score.add_part(violin);
        score.add_part(Part::with_name("Unassigned")); // no instrument set

        let instruments = score.get_instruments();
        assert_eq!(instruments.len(), 1);
        assert_eq!(instruments[0].name(), "Violin");
    }

    #[test]
    fn test_score_pad_measures() {
        let mut score = Score::new();

        let mut part1 = Part::new();
        part1.add_measure(Measure::new(1));
        part1.add_measure(Measure::new(2));

        let mut part2 = Part::new();
        part2.add_measure(Measure::new(1));

        score.add_part(part1);
        score.add_part(part2);

        score.pad_measures();

        assert_eq!(score.part(0).unwrap().num_measures(), 2);
        assert_eq!(score.part(1).unwrap().num_measures(), 2);
    }
}
