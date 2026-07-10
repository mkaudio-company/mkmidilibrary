//! Musical scale abstraction
//!
//! A `Scale` is an ordered sequence of 7 diatonic degrees built from a
//! tonic `Pitch` and a `KeyMode` (major or one of the six standard modes).
//! Mirrors a deliberately scoped subset of music21's `scale` module:
//! major/the diatonic modes only. Harmonic/melodic minor (with their
//! raised degrees) are not modeled here and are a documented follow-up,
//! not silently dropped.

use crate::core::{Accidental, Pitch, Step};

use super::KeyMode;

/// An ordered musical scale built from a tonic pitch and mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scale {
    tonic: Pitch,
    mode: KeyMode,
}

impl Scale {
    /// Create a new scale from a tonic pitch and mode.
    pub fn new(tonic: Pitch, mode: KeyMode) -> Self {
        Self { tonic, mode }
    }

    /// Build a major scale on the given tonic.
    pub fn major(tonic: Pitch) -> Self {
        Self::new(tonic, KeyMode::Major)
    }

    /// Build a natural minor (Aeolian) scale on the given tonic.
    pub fn natural_minor(tonic: Pitch) -> Self {
        Self::new(tonic, KeyMode::Minor)
    }

    /// Get the tonic pitch.
    pub fn tonic(&self) -> &Pitch {
        &self.tonic
    }

    /// Get the mode.
    pub fn mode(&self) -> KeyMode {
        self.mode
    }

    /// Semitone offsets from the tonic for each of the 7 scale degrees of a
    /// given mode. `Minor` is treated as an alias of natural-minor/Aeolian.
    pub fn intervals_for_mode(mode: KeyMode) -> [i32; 7] {
        match mode {
            KeyMode::Major => [0, 2, 4, 5, 7, 9, 11],
            KeyMode::Dorian => [0, 2, 3, 5, 7, 9, 10],
            KeyMode::Phrygian => [0, 1, 3, 5, 7, 8, 10],
            KeyMode::Lydian => [0, 2, 4, 6, 7, 9, 11],
            KeyMode::Mixolydian => [0, 2, 4, 5, 7, 9, 10],
            KeyMode::Minor | KeyMode::Aeolian => [0, 2, 3, 5, 7, 8, 10],
            KeyMode::Locrian => [0, 1, 3, 5, 6, 8, 10],
        }
    }

    /// Get all 7 pitches of this scale. Letter names cycle diatonically
    /// from the tonic's own letter (so each of the 7 natural-letter names
    /// appears exactly once), with accidentals derived from this mode's
    /// semitone pattern relative to the tonic's actual pitch class.
    pub fn pitches(&self) -> Vec<Pitch> {
        let intervals = Self::intervals_for_mode(self.mode);
        let tonic_step_index = self.tonic.step().index();
        let tonic_pc = self.tonic.pitch_class() as i32;

        (0..7)
            .map(|i| {
                let letter = Step::from_index(tonic_step_index + i as i32);
                let target_pc = (tonic_pc + intervals[i as usize]).rem_euclid(12);
                let natural_pc = letter.pitch_class() as i32;
                let alter = shortest_alter(natural_pc, target_pc);
                Pitch::from_parts(letter, None, accidental_from_alter(alter))
            })
            .collect()
    }

    /// Get the pitch for a given scale degree (1-indexed; 1 = tonic).
    /// Returns `None` for degree 0 or degree > 7.
    pub fn pitch_for_degree(&self, degree: u8) -> Option<Pitch> {
        if degree == 0 || degree > 7 {
            return None;
        }
        self.pitches().into_iter().nth(degree as usize - 1)
    }

    /// Get the scale degree (1-7) of a pitch that matches this scale's
    /// spelling (letter + accidental), if any.
    pub fn degree_of(&self, pitch: &Pitch) -> Option<u8> {
        self.pitches()
            .iter()
            .position(|p| p.step() == pitch.step() && p.accidental() == pitch.accidental())
            .map(|i| i as u8 + 1)
    }
}

/// Signed semitone alteration (in -6..=6) needed to go from `natural_pc` to
/// `target_pc` (both pitch classes 0-11), choosing the smallest-magnitude
/// representative.
fn shortest_alter(natural_pc: i32, target_pc: i32) -> i32 {
    let mut diff = (target_pc - natural_pc).rem_euclid(12);
    if diff > 6 {
        diff -= 12;
    }
    diff
}

fn accidental_from_alter(alter: i32) -> Option<Accidental> {
    match alter {
        -3 => Some(Accidental::TripleFlat),
        -2 => Some(Accidental::DoubleFlat),
        -1 => Some(Accidental::Flat),
        0 => None,
        1 => Some(Accidental::Sharp),
        2 => Some(Accidental::DoubleSharp),
        3 => Some(Accidental::TripleSharp),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(step: Step) -> Pitch {
        Pitch::from_parts(step, None, None)
    }

    #[test]
    fn test_major_scale_pitches() {
        let scale = Scale::major(c(Step::C));
        let pitches: Vec<String> = scale.pitches().iter().map(|p| p.name()).collect();
        assert_eq!(pitches, vec!["C", "D", "E", "F", "G", "A", "B"]);
    }

    #[test]
    fn test_major_scale_with_sharps() {
        // G major: F# is the only altered degree.
        let scale = Scale::major(c(Step::G));
        let pitches: Vec<String> = scale.pitches().iter().map(|p| p.name()).collect();
        assert_eq!(pitches, vec!["G", "A", "B", "C", "D", "E", "F#"]);
    }

    #[test]
    fn test_natural_minor_scale() {
        let scale = Scale::natural_minor(c(Step::A));
        let pitches: Vec<String> = scale.pitches().iter().map(|p| p.name()).collect();
        assert_eq!(pitches, vec!["A", "B", "C", "D", "E", "F", "G"]);
    }

    #[test]
    fn test_pitch_for_degree_and_degree_of() {
        let scale = Scale::major(c(Step::G));
        let fifth = scale.pitch_for_degree(5).unwrap();
        assert_eq!(fifth.name(), "D");

        let leading_tone = scale.pitch_for_degree(7).unwrap();
        assert_eq!(leading_tone.name(), "F#");
        assert_eq!(scale.degree_of(&leading_tone), Some(7));

        assert_eq!(scale.pitch_for_degree(0), None);
        assert_eq!(scale.pitch_for_degree(8), None);
    }

    #[test]
    fn test_dorian_mode() {
        // D dorian == white keys starting on D (same pitches as C major).
        let scale = Scale::new(c(Step::D), KeyMode::Dorian);
        let pitches: Vec<String> = scale.pitches().iter().map(|p| p.name()).collect();
        assert_eq!(pitches, vec!["D", "E", "F", "G", "A", "B", "C"]);
    }
}
