//! Part representation
//!
//! A Part represents a single instrument part in a score.

use std::fmt;

use super::measure::Measure;

/// Instrument information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instrument {
    /// Instrument name
    name: String,
    /// Abbreviated name
    abbreviation: Option<String>,
    /// MIDI program number (0-127)
    midi_program: u8,
    /// MIDI channel (0-15)
    midi_channel: Option<u8>,
    /// Transposition in semitones
    transposition: i8,
}

impl Instrument {
    /// Create a new instrument
    pub fn new(name: impl Into<String>, midi_program: u8) -> Self {
        Self {
            name: name.into(),
            abbreviation: None,
            midi_program,
            midi_channel: None,
            transposition: 0,
        }
    }

    /// Create a piano
    pub fn piano() -> Self {
        Self::new("Piano", 0)
    }

    /// Create a violin
    pub fn violin() -> Self {
        Self::new("Violin", 40)
    }

    /// Create a flute
    pub fn flute() -> Self {
        Self::new("Flute", 73)
    }

    /// Create a trumpet
    pub fn trumpet() -> Self {
        let mut inst = Self::new("Trumpet", 56);
        inst.transposition = -2; // Bb trumpet
        inst
    }

    /// Get the name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Set the name
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// Get the abbreviation
    pub fn abbreviation(&self) -> Option<&str> {
        self.abbreviation.as_deref()
    }

    /// Set the abbreviation
    pub fn set_abbreviation(&mut self, abbr: impl Into<String>) {
        self.abbreviation = Some(abbr.into());
    }

    /// Get the MIDI program number
    pub fn midi_program(&self) -> u8 {
        self.midi_program
    }

    /// Set the MIDI program number
    pub fn set_midi_program(&mut self, program: u8) {
        self.midi_program = program;
    }

    /// Get the MIDI channel
    pub fn midi_channel(&self) -> Option<u8> {
        self.midi_channel
    }

    /// Set the MIDI channel
    pub fn set_midi_channel(&mut self, channel: u8) {
        self.midi_channel = Some(channel);
    }

    /// Get the transposition
    pub fn transposition(&self) -> i8 {
        self.transposition
    }

    /// Set the transposition
    pub fn set_transposition(&mut self, semitones: i8) {
        self.transposition = semitones;
    }
}

impl Default for Instrument {
    fn default() -> Self {
        Self::piano()
    }
}

/// A single instrument part
#[derive(Debug, Clone, Default)]
pub struct Part {
    /// Part name
    name: Option<String>,
    /// Abbreviated name
    abbreviation: Option<String>,
    /// Instrument
    instrument: Option<Instrument>,
    /// Measures in this part
    measures: Vec<Measure>,
    /// Part ID
    id: Option<String>,
}

impl Part {
    /// Create a new empty part
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a part with a name
    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..Default::default()
        }
    }

    /// Get the name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Set the name
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Get the abbreviation
    pub fn abbreviation(&self) -> Option<&str> {
        self.abbreviation.as_deref()
    }

    /// Set the abbreviation
    pub fn set_abbreviation(&mut self, abbr: impl Into<String>) {
        self.abbreviation = Some(abbr.into());
    }

    /// Get the instrument
    pub fn instrument(&self) -> Option<&Instrument> {
        self.instrument.as_ref()
    }

    /// Set the instrument
    pub fn set_instrument(&mut self, instrument: Instrument) {
        self.instrument = Some(instrument);
    }

    /// Get the part ID
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// Set the part ID
    pub fn set_id(&mut self, id: impl Into<String>) {
        self.id = Some(id.into());
    }

    /// Get all measures
    pub fn measures(&self) -> &[Measure] {
        &self.measures
    }

    /// Get mutable measures
    pub fn measures_mut(&mut self) -> &mut Vec<Measure> {
        &mut self.measures
    }

    /// Get a specific measure
    pub fn measure(&self, index: usize) -> Option<&Measure> {
        self.measures.get(index)
    }

    /// Get a mutable specific measure
    pub fn measure_mut(&mut self, index: usize) -> Option<&mut Measure> {
        self.measures.get_mut(index)
    }

    /// Get measure by number
    pub fn measure_by_number(&self, number: u32) -> Option<&Measure> {
        self.measures.iter().find(|m| m.number() == number)
    }

    /// Add a measure
    pub fn add_measure(&mut self, measure: Measure) {
        self.measures.push(measure);
    }

    /// Insert a measure at index
    pub fn insert_measure(&mut self, index: usize, measure: Measure) {
        self.measures.insert(index, measure);
    }

    /// Remove a measure
    pub fn remove_measure(&mut self, index: usize) -> Option<Measure> {
        if index < self.measures.len() {
            Some(self.measures.remove(index))
        } else {
            None
        }
    }

    /// Get the number of measures
    pub fn num_measures(&self) -> usize {
        self.measures.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.measures.is_empty()
    }

    /// Clear all measures
    pub fn clear(&mut self) {
        self.measures.clear();
    }

    /// Get total duration in quarter lengths
    pub fn duration(&self) -> crate::core::Fraction {
        self.measures.iter().map(|m| m.duration()).sum()
    }

    /// Iterate over all notes in the part
    pub fn notes(&self) -> impl Iterator<Item = &crate::core::Note> {
        self.measures.iter().flat_map(|m| m.notes())
    }

    /// Iterate over all chords in the part
    pub fn chords(&self) -> impl Iterator<Item = &crate::core::Chord> {
        self.measures.iter().flat_map(|m| m.chords())
    }

    /// Create measures up to a given number if they don't exist
    pub fn ensure_measures(&mut self, count: usize) {
        while self.measures.len() < count {
            let num = self.measures.len() as u32 + 1;
            self.measures.push(Measure::new(num));
        }
    }

    /// Renumber measures starting from 1
    pub fn renumber_measures(&mut self) {
        let has_pickup = self.measures.first().map(|m| m.is_pickup()).unwrap_or(false);
        let start = if has_pickup { 0 } else { 1 };

        for (i, measure) in self.measures.iter_mut().enumerate() {
            if i == 0 && has_pickup {
                measure.set_number(0);
            } else {
                measure.set_number(start + i as u32 - if has_pickup { 0 } else { 0 });
            }
        }
    }
}

impl fmt::Display for Part {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.name.as_deref().unwrap_or("Unnamed Part");
        write!(f, "Part '{}' ({} measures)", name, self.measures.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_creation() {
        let part = Part::with_name("Violin I");
        assert_eq!(part.name(), Some("Violin I"));
        assert!(part.is_empty());
    }

    #[test]
    fn test_part_measures() {
        let mut part = Part::new();
        part.add_measure(Measure::new(1));
        part.add_measure(Measure::new(2));

        assert_eq!(part.num_measures(), 2);
        assert_eq!(part.measure(0).unwrap().number(), 1);
    }

    #[test]
    fn test_part_instrument() {
        let mut part = Part::with_name("Piano");
        part.set_instrument(Instrument::piano());

        assert_eq!(part.instrument().unwrap().midi_program(), 0);
    }

    #[test]
    fn test_instrument_creation() {
        let trumpet = Instrument::trumpet();
        assert_eq!(trumpet.name(), "Trumpet");
        assert_eq!(trumpet.transposition(), -2);
    }

    #[test]
    fn test_ensure_measures() {
        let mut part = Part::new();
        part.ensure_measures(5);

        assert_eq!(part.num_measures(), 5);
        assert_eq!(part.measure(4).unwrap().number(), 5);
    }
}
