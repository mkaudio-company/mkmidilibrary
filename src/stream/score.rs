//! Score representation
//!
//! A Score is a complete musical composition containing multiple parts.

use std::fmt;

use crate::notation::{KeySignature, Tempo, TimeSignature};

use super::part::Part;

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
        self.parts.iter().map(|p| p.num_measures()).max().unwrap_or(0)
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
