//! Chord representation
//!
//! A Chord represents multiple simultaneous pitches with a shared duration.

use std::cmp::Ordering;
use std::fmt;

use super::{Duration, Fraction, Interval, Note, Pitch};

/// Chord quality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChordQuality {
    Major,
    Minor,
    Diminished,
    Augmented,
    Dominant,
    HalfDiminished,
    FullyDiminished,
    Suspended2,
    Suspended4,
    Power,
    Other,
}

impl ChordQuality {
    /// Get the symbol for this quality
    pub fn symbol(&self) -> &'static str {
        match self {
            ChordQuality::Major => "",
            ChordQuality::Minor => "m",
            ChordQuality::Diminished => "dim",
            ChordQuality::Augmented => "aug",
            ChordQuality::Dominant => "7",
            ChordQuality::HalfDiminished => "m7b5",
            ChordQuality::FullyDiminished => "dim7",
            ChordQuality::Suspended2 => "sus2",
            ChordQuality::Suspended4 => "sus4",
            ChordQuality::Power => "5",
            ChordQuality::Other => "",
        }
    }
}

/// A chord (multiple simultaneous pitches)
#[derive(Debug, Clone, PartialEq)]
pub struct Chord {
    /// The notes in this chord (each has its own pitch)
    notes: Vec<Note>,
    /// Shared duration for the chord
    duration: Duration,
    /// Offset within the stream
    offset: Fraction,
}

impl Chord {
    /// Create a new chord from notes
    pub fn new(notes: Vec<Note>, duration: Duration) -> Self {
        Self {
            notes,
            duration,
            offset: Fraction::new(0, 1),
        }
    }

    /// Create a chord from pitches
    pub fn from_pitches(pitches: Vec<Pitch>, duration: Duration) -> Self {
        let notes = pitches
            .into_iter()
            .map(|p| Note::new(p, duration.clone()))
            .collect();
        Self {
            notes,
            duration,
            offset: Fraction::new(0, 1),
        }
    }

    /// Create a chord from pitch strings
    pub fn from_pitch_strings(pitches: &[&str], duration: Duration) -> Result<Self, super::ParseError> {
        let parsed: Result<Vec<Pitch>, _> = pitches.iter().map(|s| s.parse()).collect();
        Ok(Self::from_pitches(parsed?, duration))
    }

    /// Create a major triad
    pub fn major_triad(root: Pitch) -> Self {
        let third = root.transpose(&Interval::major_third());
        let fifth = root.transpose(&Interval::perfect_fifth());
        Self::from_pitches(vec![root, third, fifth], Duration::quarter())
    }

    /// Create a minor triad
    pub fn minor_triad(root: Pitch) -> Self {
        let third = root.transpose(&Interval::minor_third());
        let fifth = root.transpose(&Interval::perfect_fifth());
        Self::from_pitches(vec![root, third, fifth], Duration::quarter())
    }

    /// Create a diminished triad
    pub fn diminished_triad(root: Pitch) -> Self {
        let third = root.transpose(&Interval::minor_third());
        let fifth = root.transpose(&Interval::tritone());
        Self::from_pitches(vec![root, third, fifth], Duration::quarter())
    }

    /// Create an augmented triad
    pub fn augmented_triad(root: Pitch) -> Self {
        let third = root.transpose(&Interval::major_third());
        let fifth = root.transpose(&Interval::new(4, 8)); // augmented fifth
        Self::from_pitches(vec![root, third, fifth], Duration::quarter())
    }

    /// Create a dominant seventh chord
    pub fn dominant_seventh(root: Pitch) -> Self {
        let third = root.transpose(&Interval::major_third());
        let fifth = root.transpose(&Interval::perfect_fifth());
        let seventh = root.transpose(&Interval::minor_seventh());
        Self::from_pitches(vec![root, third, fifth, seventh], Duration::quarter())
    }

    /// Create a major seventh chord
    pub fn major_seventh(root: Pitch) -> Self {
        let third = root.transpose(&Interval::major_third());
        let fifth = root.transpose(&Interval::perfect_fifth());
        let seventh = root.transpose(&Interval::major_seventh());
        Self::from_pitches(vec![root, third, fifth, seventh], Duration::quarter())
    }

    /// Create a minor seventh chord
    pub fn minor_seventh(root: Pitch) -> Self {
        let third = root.transpose(&Interval::minor_third());
        let fifth = root.transpose(&Interval::perfect_fifth());
        let seventh = root.transpose(&Interval::minor_seventh());
        Self::from_pitches(vec![root, third, fifth, seventh], Duration::quarter())
    }

    /// Get the notes
    pub fn notes(&self) -> &[Note] {
        &self.notes
    }

    /// Get mutable notes
    pub fn notes_mut(&mut self) -> &mut Vec<Note> {
        &mut self.notes
    }

    /// Get the pitches
    pub fn pitches(&self) -> Vec<&Pitch> {
        self.notes.iter().map(|n| n.pitch()).collect()
    }

    /// Get pitch classes (0-11)
    pub fn pitch_classes(&self) -> Vec<u8> {
        self.notes.iter().map(|n| n.pitch().pitch_class()).collect()
    }

    /// Get ordered pitch classes (sorted, unique)
    pub fn ordered_pitch_classes(&self) -> Vec<u8> {
        let mut pcs = self.pitch_classes();
        pcs.sort();
        pcs.dedup();
        pcs
    }

    /// Add a note to the chord
    pub fn add(&mut self, note: Note) {
        self.notes.push(note);
    }

    /// Add a pitch to the chord
    pub fn add_pitch(&mut self, pitch: Pitch) {
        self.notes.push(Note::new(pitch, self.duration.clone()));
    }

    /// Remove a note from the chord
    pub fn remove(&mut self, index: usize) -> Option<Note> {
        if index < self.notes.len() {
            Some(self.notes.remove(index))
        } else {
            None
        }
    }

    /// Get the duration
    pub fn duration(&self) -> &Duration {
        &self.duration
    }

    /// Set the duration
    pub fn set_duration(&mut self, duration: Duration) {
        self.duration = duration.clone();
        for note in &mut self.notes {
            note.set_duration(duration.clone());
        }
    }

    /// Get the offset
    pub fn offset(&self) -> Fraction {
        self.offset
    }

    /// Set the offset
    pub fn set_offset(&mut self, offset: Fraction) {
        self.offset = offset;
    }

    /// Get the quarter length
    pub fn quarter_length(&self) -> Fraction {
        self.duration.quarter_length()
    }

    /// Get the bass (lowest pitch)
    pub fn bass(&self) -> Option<&Pitch> {
        self.notes.iter().map(|n| n.pitch()).min()
    }

    /// Get the root (may differ from bass in inversions)
    pub fn root(&self) -> Option<Pitch> {
        // Simple root detection based on stacked thirds
        // This is a simplified version - full implementation would use more sophisticated analysis
        if self.notes.is_empty() {
            return None;
        }

        let pcs = self.ordered_pitch_classes();
        if pcs.len() < 2 {
            return self.bass().cloned();
        }

        // Try each pitch as potential root
        for &pc in &pcs {
            let normalized: Vec<u8> = pcs.iter().map(|&p| (p + 12 - pc) % 12).collect();

            // Check for common chord patterns (thirds-based)
            if normalized.contains(&4) || normalized.contains(&3) {
                // Has a third
                if normalized.contains(&7) {
                    // Has a fifth - this is likely the root
                    return self.notes.iter()
                        .find(|n| n.pitch().pitch_class() == pc)
                        .map(|n| n.pitch().clone());
                }
            }
        }

        // Default to bass
        self.bass().cloned()
    }

    /// Get the inversion (0 = root position, 1 = first inversion, etc.)
    pub fn inversion(&self) -> u8 {
        if let (Some(bass), Some(root)) = (self.bass(), self.root()) {
            let bass_pc = bass.pitch_class();
            let root_pc = root.pitch_class();

            if bass_pc == root_pc {
                return 0;
            }

            let pcs = self.ordered_pitch_classes();
            let root_idx = pcs.iter().position(|&pc| pc == root_pc).unwrap_or(0);
            let bass_idx = pcs.iter().position(|&pc| pc == bass_pc).unwrap_or(0);

            ((bass_idx as i32 - root_idx as i32).rem_euclid(pcs.len() as i32)) as u8
        } else {
            0
        }
    }

    /// Determine the chord quality
    pub fn quality(&self) -> ChordQuality {
        if self.notes.len() < 2 {
            return ChordQuality::Other;
        }

        // Use the first note (bass) as the root for interval calculation
        let root_pc = self.notes[0].pitch().pitch_class();

        // Get unique pitch classes
        let mut pcs: Vec<u8> = self.pitch_classes();
        pcs.sort();
        pcs.dedup();

        // Calculate intervals from root
        let mut intervals: Vec<u8> = pcs.iter().map(|&pc| (pc + 12 - root_pc) % 12).collect();
        intervals.sort();

        // Check for common chord types
        match intervals.as_slice() {
            // Triads
            [0, 4, 7] => ChordQuality::Major,
            [0, 3, 7] => ChordQuality::Minor,
            [0, 3, 6] => ChordQuality::Diminished,
            [0, 4, 8] => ChordQuality::Augmented,
            [0, 2, 7] => ChordQuality::Suspended2,
            [0, 5, 7] => ChordQuality::Suspended4,
            [0, 7] => ChordQuality::Power,
            // Seventh chords
            [0, 4, 7, 10] => ChordQuality::Dominant,
            [0, 4, 7, 11] => ChordQuality::Major,
            [0, 3, 7, 10] => ChordQuality::Minor,
            [0, 3, 6, 10] => ChordQuality::HalfDiminished,
            [0, 3, 6, 9] => ChordQuality::FullyDiminished,
            _ => ChordQuality::Other,
        }
    }

    /// Check if this is a major triad
    pub fn is_major_triad(&self) -> bool {
        self.quality() == ChordQuality::Major && self.notes.len() == 3
    }

    /// Check if this is a minor triad
    pub fn is_minor_triad(&self) -> bool {
        self.quality() == ChordQuality::Minor && self.notes.len() == 3
    }

    /// Check if this is a diminished triad
    pub fn is_diminished_triad(&self) -> bool {
        self.quality() == ChordQuality::Diminished && self.notes.len() == 3
    }

    /// Check if this is an augmented triad
    pub fn is_augmented_triad(&self) -> bool {
        self.quality() == ChordQuality::Augmented && self.notes.len() == 3
    }

    /// Check if this is a dominant seventh
    pub fn is_dominant_seventh(&self) -> bool {
        self.quality() == ChordQuality::Dominant && self.notes.len() == 4
    }

    /// Get the third of the chord (if present)
    pub fn third(&self) -> Option<&Pitch> {
        self.get_chord_step(3)
    }

    /// Get the fifth of the chord (if present)
    pub fn fifth(&self) -> Option<&Pitch> {
        self.get_chord_step(5)
    }

    /// Get the seventh of the chord (if present)
    pub fn seventh(&self) -> Option<&Pitch> {
        self.get_chord_step(7)
    }

    /// Get a chord step (1 = root, 3 = third, 5 = fifth, etc.)
    pub fn get_chord_step(&self, step: u8) -> Option<&Pitch> {
        let root = self.root()?;
        let root_pc = root.pitch_class();

        // Calculate expected pitch class for the step
        let step_semitones = match step {
            1 => 0,
            2 => 2,
            3 => 3, // could be 3 (minor) or 4 (major)
            4 => 5,
            5 => 7,
            6 => 9,
            7 => 10, // could be 10 (minor) or 11 (major)
            _ => return None,
        };

        // Find pitch closest to expected
        for note in &self.notes {
            let pc = note.pitch().pitch_class();
            let interval = (pc + 12 - root_pc) % 12;

            // Allow for major/minor variants
            if step == 3 && (interval == 3 || interval == 4) {
                return Some(note.pitch());
            }
            if step == 7 && (interval == 10 || interval == 11) {
                return Some(note.pitch());
            }
            if interval == step_semitones {
                return Some(note.pitch());
            }
        }

        None
    }

    /// Transpose the chord
    pub fn transpose(&self, interval: &Interval) -> Chord {
        let notes = self.notes.iter().map(|n| n.transpose(interval)).collect();
        Chord {
            notes,
            duration: self.duration.clone(),
            offset: self.offset,
        }
    }

    /// Transpose by semitones
    pub fn transpose_semitones(&self, semitones: i32) -> Chord {
        let notes = self.notes.iter().map(|n| n.transpose_semitones(semitones)).collect();
        Chord {
            notes,
            duration: self.duration.clone(),
            offset: self.offset,
        }
    }

    /// Get chord symbol (e.g., "Cmaj7", "Dm", "G7")
    pub fn symbol(&self) -> String {
        if let Some(root) = self.root() {
            format!("{}{}", root.name(), self.quality().symbol())
        } else {
            "?".to_string()
        }
    }

    /// Check if the chord contains a specific pitch class
    pub fn contains_pitch_class(&self, pc: u8) -> bool {
        self.pitch_classes().contains(&pc)
    }

    /// Get the interval from bass to root
    pub fn bass_to_root_interval(&self) -> Option<Interval> {
        let bass = self.bass()?;
        let root = self.root()?;

        let semitones = (root.pitch_class() as i32 - bass.pitch_class() as i32).rem_euclid(12);
        Some(Interval::from(semitones))
    }
}

impl Default for Chord {
    fn default() -> Self {
        Self::new(Vec::new(), Duration::quarter())
    }
}

impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pitches: Vec<String> = self.notes.iter().map(|n| n.name()).collect();
        write!(f, "<{}>", pitches.join(" "))
    }
}

impl PartialOrd for Chord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Compare by lowest pitch
        match (self.bass(), other.bass()) {
            (Some(a), Some(b)) => a.partial_cmp(b),
            (Some(_), None) => Some(Ordering::Greater),
            (None, Some(_)) => Some(Ordering::Less),
            (None, None) => Some(Ordering::Equal),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Step;

    #[test]
    fn test_chord_major_triad() {
        let c = Pitch::from_parts(Step::C, Some(4), None);
        let chord = Chord::major_triad(c);

        assert_eq!(chord.notes.len(), 3);
        assert!(chord.is_major_triad());
        assert_eq!(chord.quality(), ChordQuality::Major);
    }

    #[test]
    fn test_chord_minor_triad() {
        let a = Pitch::from_parts(Step::A, Some(4), None);
        let chord = Chord::minor_triad(a);

        assert!(chord.is_minor_triad());
        assert_eq!(chord.quality(), ChordQuality::Minor);
    }

    #[test]
    fn test_chord_from_strings() {
        let chord = Chord::from_pitch_strings(&["C4", "E4", "G4"], Duration::quarter()).unwrap();
        assert!(chord.is_major_triad());
    }

    #[test]
    fn test_chord_bass_and_root() {
        let c = Pitch::from_parts(Step::C, Some(4), None);
        let chord = Chord::major_triad(c.clone());

        assert_eq!(chord.bass().unwrap().step(), Step::C);
        assert_eq!(chord.root().unwrap().step(), Step::C);
    }

    #[test]
    fn test_chord_dominant_seventh() {
        let g = Pitch::from_parts(Step::G, Some(4), None);
        let chord = Chord::dominant_seventh(g);

        assert_eq!(chord.notes.len(), 4);
        assert!(chord.is_dominant_seventh());
    }

    #[test]
    fn test_chord_transpose() {
        let c = Pitch::from_parts(Step::C, Some(4), None);
        let chord = Chord::major_triad(c);
        let transposed = chord.transpose(&Interval::perfect_fifth());

        assert_eq!(transposed.root().unwrap().step(), Step::G);
    }
}
